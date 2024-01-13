from re import sub
from shutil import rmtree
import shutil
from pathlib import Path
from typing import List

from py import test
from ultralytics import YOLO
from env_cfg import EnvCfg
from PIL import Image
# Runs in the ultralytics container
# switch function in main
# Runs 'detect' finding the cards in the screenshot

# Run split_train_validate 1st, which also collapses the classes into one class, a card


cfg = EnvCfg()


def read_classes() -> List[str]:
    with open(cfg.CARD_YOLO_PATH / "classes.txt") as f:
        classes = f.read().strip().split("\n")

    if len(classes) != 52:
        raise Exception(f"Expected 52 classes, was {len(classes)}")
    
    return classes
    

def prepare_data(valid_percent: float=0.0, test_percent: float =0.35):
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
            center_x = float(fields[1])
            center_y = float(fields[2])
            width = float(fields[3])
            height = float(fields[4])

            # convert to x_min, y_min, x_max, y_max
            center_x = center_x * img.width
            center_y = center_y * img.height
            width = width * img.width
            height = height * img.height

            x_min = center_x - width / 2
            x_max = center_x + width / 2
            y_min = center_y - height / 2
            y_max = center_y + height / 2

            # open the png image and crop this box out
            # Crop the image to the specified rectangle

            cropped_img = img.crop((x_min, y_min, x_max, y_max))

            # Resize the image to 128x128

            cropped_img = cropped_img.resize((cfg.CLASSIFY_IMG_SZ, cfg.CLASSIFY_IMG_SZ))

            # Save the cropped image
            target_path = cfg.CLASSIFY_DATA_PATH / cfg.TRAIN_FOLDER_NAME / card_class / f"{image_num}.png"
            image_num += 1
            target_path.parent.mkdir(parents=True, exist_ok=True)
            cropped_img.save(target_path)

            print(f"Saved {target_path}, dimensions {cropped_img.width}x{cropped_img.height}")
            
            cropped_img.close()
        # Close the original and cropped images
        img.close()

    # now move some to test & valid from the training folder
    for class_dir in (cfg.CLASSIFY_DATA_PATH / cfg.TRAIN_FOLDER_NAME).iterdir():
        if not class_dir.is_dir():
            continue

        test_dir = cfg.CLASSIFY_DATA_PATH / cfg.TEST_FOLDER_NAME / class_dir.name
        test_dir.mkdir(parents=True, exist_ok=True)

        valid_dir = cfg.CLASSIFY_DATA_PATH / cfg.VALID_FOLDER_NAME / class_dir.name
        valid_dir.mkdir(parents=True, exist_ok=True)

        train_files = [f for f in class_dir.iterdir()]
        num_test = int(len(train_files) * test_percent)
        test_files = train_files[:num_test]
        num_valid = int(len(train_files) * valid_percent)
        valid_files = train_files[num_test:num_test + num_valid]

        train_files = train_files[num_valid+num_test:]

        if test_percent > 0 and len(test_files) == 0:
            raise Exception(f"Not enough files for test in {class_dir}")
        if valid_percent > 0 and len(valid_files) == 0:
            raise Exception(f"Not enough files for valid in {class_dir}")
        if len(train_files) == 0:
            raise Exception(f"Not enough files for train in {class_dir}")

        for f in test_files:
            shutil.move(f, test_dir)

        for f in valid_files:
            shutil.move(f, valid_dir)


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
    prepare_data()
    count_instances_per_class()
    
    clean_run_dir()
    train()
    
    # clean_run_dir()
    # train()    
    predict()