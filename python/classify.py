from re import sub
from shutil import rmtree
import shutil
from pathlib import Path

from py import test
from ultralytics import YOLO
from env_cfg import EnvCfg
from PIL import Image
# Runs in the ultralytics container
# switch function in main
# Runs 'detect' finding the cards in the screenshot

# Run split_train_validate 1st, which also collapses the classes into one class, a card

from scratch_cnn import read_classes, count_instances_per_class

cfg = EnvCfg()


def prepare_data(test_percent=0.35):
    """
    https://docs.ultralytics.com/datasets/classify/#dataset-format
    We need a train/test split

    /train/[class]/[image_num].png
    /test/[class]/[image_num].png
    """
    print(f"Converting yolo dataset in [{cfg.CARD_YOLO_PATH}] to classification format in [{cfg.CLASSIFY_DATA_PATH}]")

    if cfg.CLASSIFY_DATA_PATH.exists():
        print(f"Removing {cfg.CLASSIFY_DATA_PATH}")
        shutil.rmtree(cfg.CLASSIFY_DATA_PATH)

    images_dir = cfg.CARD_YOLO_PATH / cfg.IMAGE_FOLDER_NAME
    labels_dir = cfg.CARD_YOLO_PATH / cfg.LABEL_FOLDER_NAME

    image_num = 0

    classes = read_classes()

    # first put everything into train

    for label_file in labels_dir.iterdir():
        if label_file.suffix != ".txt":
            continue

        image_file = images_dir / label_file.with_suffix(".png").name

        img = Image.open(image_file)

        for line in label_file.open("r").readlines():
            fields = line.split(" ")
            card_class = classes[int(fields[0])]
            xs = []
            ys = []
            for i in range(0, 4):
                xs.append(float(fields[1 + i * 2]))
                ys.append(float(fields[2 + i * 2]))

            x_min = min(xs) 
            x_max = max(xs)
            y_min = min(ys)
            y_max = max(ys)

            width = x_max - x_min
            height = y_max - y_min

            x_min = x_min * img.width
            x_max = x_max * img.width
            y_min = y_min * img.height
            y_max = y_max * img.height

            # open the png image and crop this box out
            # Crop the image to the specified rectangle


            cropped_img = img.crop((x_min, y_min, x_max, y_max))

            # Resize the image to 128x128

            cropped_img = cropped_img.resize((128, 128))

            # Save the cropped image
            target_path = cfg.CLASSIFY_DATA_PATH / cfg.TRAIN_FOLDER_NAME / card_class / f"{image_num}.png"
            image_num += 1
            target_path.parent.mkdir(parents=True, exist_ok=True)
            cropped_img.save(target_path)

            print(f"Saved {target_path}, dimensions {cropped_img.width}x{cropped_img.height}")
            
            cropped_img.close()
        # Close the original and cropped images
        img.close()

    # now move some to test
    for class_dir in (cfg.CLASSIFY_DATA_PATH / cfg.TRAIN_FOLDER_NAME).iterdir():
        if not class_dir.is_dir():
            continue

        test_dir = cfg.CLASSIFY_DATA_PATH / cfg.TEST_FOLDER_NAME / class_dir.name
        test_dir.mkdir(parents=True, exist_ok=True)

        train_files = [f for f in class_dir.iterdir()]
        num_test = int(len(train_files) * test_percent)
        test_files = train_files[:num_test]
        train_files = train_files[num_test:]

        for f in test_files:
            shutil.move(f, test_dir)


def count_instances_per_class():
    
    classes = read_classes()
    
    # now count how many of each class we have
    for class_name in classes:
        train_dir = cfg.CLASSIFY_DATA_PATH / cfg.TRAIN_FOLDER_NAME / class_name
        num_train_files = len(list(train_dir.iterdir()))

        if num_train_files < 2:
            print(f"Not enough training files for {class_name}, only {num_train_files}")

        test_dir = cfg.CLASSIFY_DATA_PATH / cfg.TEST_FOLDER_NAME / class_name
        num_test_files = len(list(test_dir.iterdir()))

        if num_test_files < 1:
            print(f"Not enough test files for {class_name}, only {num_test_files}")

        


def train():
    
    
    # Load a model
    model = YOLO('yolov8n-cls.pt')  # load a pretrained model (recommended for training)

    # Train the model
    model.train(
        data=cfg.CLASSIFY_DATA_PATH, epochs=30, imgsz=128, degrees=45,
        project=cfg.CLASSIFY_PROJECT_PATH,
        name=cfg.CLASSIFY_MODEL_NAME
    )
    

def predict():

    model = YOLO(cfg.CLASSIFY_PROJECT_PATH / cfg.CLASSIFY_MODEL_NAME / 'weights/best.pt')

    predict_dir = cfg.PYTHON_SRC_DIR / "predictions"

    image_dir = cfg.CLASSIFY_DATA_PATH / cfg.TEST_FOLDER_NAME 

    if predict_dir.exists():
        shutil.rmtree(predict_dir)

    predict_dir.mkdir(parents=True, exist_ok=True)

    for image_file in image_dir.rglob("*.png"):
        if image_file.suffix != ".png":
            continue

        correct_class=image_file.parent.name

        # not really needed, predictions are already put in the RUNS_DIR/prediction
        target_path = predict_dir / image_file.name
        shutil.copy(image_file, target_path)

        print(f"Predicting {image_file} to {target_path}")
        results = model.predict(
            target_path, conf=0.15, 
            project=cfg.CLASSIFY_PROJECT_PATH,
            # save=True, 
            imgsz=cfg.CLASSIFY_IMG_SZ)
        
        result = results[0]
        names = result.names
        top_5 = [names[c] for c in result.probs.top5]
        top_5_conf = [100 * c for c in result.probs.top5conf.tolist()]
        print(f"Result [Correct: {correct_class}]:\n{top_5}\n{top_5_conf}")

    shutil.rmtree(predict_dir)

def clean_run_dir():
    for sub_dir in cfg.CLASSIFY_PROJECT_PATH.iterdir():
        if sub_dir.is_dir():
            rmtree(sub_dir)
        else:
            sub_dir.unlink()

if __name__ == "__main__":
    # prepare_data()
    # count_instances_per_class()
    
    #clean_run_dir()
    # train()
    
    # clean_run_dir()
    # train()    
    predict()