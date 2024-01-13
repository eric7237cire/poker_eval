from re import split, sub
from shutil import rmtree
import shutil
from pathlib import Path
from ultralytics import YOLO
from env_cfg import EnvCfg
import random
# Runs in the ultralytics container
# switch function in main
# Runs 'detect' finding the cards in the screenshot

# Run split_train_validate 1st, which also collapses the classes into one class, a card



# python /eric/python/train.py

cfg = EnvCfg()

def do_split(validation_split = 0.2, collapse_to_one_class = False) :

    print(f"Cleaning {cfg.DETECT_DATA_PATH}")
    shutil.rmtree(cfg.DETECT_DATA_PATH)

    image_files = [f for f in (cfg.CARD_YOLO_PATH / cfg.IMAGE_FOLDER_NAME).iterdir()]
    label_files = [f for f in (cfg.CARD_YOLO_PATH / cfg.LABEL_FOLDER_NAME).iterdir()]
    
    print(f"Found {len(image_files)} images and {len(label_files)} labels")

    # sanity check that the files are the same
    check_images = set([f.stem for f in image_files])
    check_labels = set([f.stem for f in label_files])

    if check_images != check_labels :
        print("Images and labels do not match")
        print(check_images - check_labels)
        print(check_labels - check_images)
        return
    
    # randomly shuffle
    random.shuffle(image_files)

    # split
    num_validate = int(len(image_files) * validation_split)

    validate_files = image_files[:num_validate]
    train_files = image_files[num_validate:]

    print(f"Split into {len(validate_files)} validation files and {len(train_files)} training files")

    target_train_dir = cfg.DETECT_DATA_PATH / cfg.TRAIN_FOLDER_NAME
    target_validate_dir = cfg.DETECT_DATA_PATH / cfg.VALID_FOLDER_NAME

    for dir_base in [target_train_dir, target_validate_dir] :
        for dir_name in [cfg.IMAGE_FOLDER_NAME, cfg.LABEL_FOLDER_NAME] :
            (dir_base / dir_name).mkdir(parents=True, exist_ok=True)
    
    # copy files
    for f in validate_files :
        shutil.copy(f, target_validate_dir / cfg.IMAGE_FOLDER_NAME)
        label_file = (cfg.CARD_YOLO_PATH /  cfg.LABEL_FOLDER_NAME / f.stem).with_suffix(".txt")
        target_label_path = target_validate_dir /  cfg.LABEL_FOLDER_NAME
        shutil.copy(label_file, target_label_path)

        if collapse_to_one_class :
            replace_with_one_class(target_label_path / label_file.name)

    for f in train_files :
        shutil.copy(f, target_train_dir / cfg.IMAGE_FOLDER_NAME)
        label_file = (cfg.CARD_YOLO_PATH /  cfg.LABEL_FOLDER_NAME / f.stem).with_suffix(".txt")
        target_label_path = target_train_dir /  cfg.LABEL_FOLDER_NAME
        shutil.copy(label_file, target_label_path)

        if collapse_to_one_class :
            replace_with_one_class(target_label_path / label_file.name)

def replace_with_one_class(txt_file: Path):
    with open(txt_file, "r") as f:
        lines = f.readlines()
    
    with open(txt_file, "w") as f:
        for line in lines:
            fields = line.split(" ")
            fields[0] = "0"
            f.write(" ".join(fields))



def train():
    
    
    # Load the model.
    model = YOLO(cfg.PYTHON_SRC_DIR / 'yolov8n.pt')
    
    # Training.
    results = model.train(
        data=cfg.PYTHON_SRC_DIR / 'zynga_1.yml',
        imgsz=cfg.DETECT_IMG_SZ,
        epochs=100,
        batch=4,
        degrees=45,
        name=cfg.DETECT_MODEL_NAME)

def predict():

    model = YOLO(cfg.RUNS_DIR / 'detect' / cfg.DETECT_MODEL_NAME / 'weights/best.pt')

    predict_dir = cfg.PYTHON_SRC_DIR / "predictions"

    image_dir = Path("/usr/src/datasets/zynga/valid/images/")

    if predict_dir.exists():
        shutil.rmtree(predict_dir)

    predict_dir.mkdir(parents=True, exist_ok=True)

    for image_file in image_dir.iterdir():
        if image_file.suffix != ".png":
            continue

        # not really needed, predictions are already put in the RUNS_DIR/prediction
        target_path = predict_dir / image_file.name
        shutil.copy(image_file, target_path)

        print(f"Predicting {image_file} to {target_path}")
        model.predict(target_path, conf=0.15, save=True, imgsz=cfg.DETECT_IMG_SZ)

    shutil.rmtree(predict_dir)

def clean_run_dir():
    for sub_dir in (cfg.RUNS_DIR / "detect").iterdir():
        if sub_dir.is_dir():
            rmtree(sub_dir)
        else:
            sub_dir.unlink()

if __name__ == "__main__":
    do_split(0.2, True)
    clean_run_dir()
    train()    
    predict()