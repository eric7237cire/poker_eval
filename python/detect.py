from re import split, sub
from shutil import rmtree
import shutil
from pathlib import Path
import sys
from typing import Dict, List, Set
from ultralytics import YOLO
import yaml
from env_cfg import EnvCfg
import random

from classify import get_class_map, read_classes, get_card_classes
# Runs in the ultralytics container
# switch function in main
# Runs 'detect' finding the cards in the screenshot

# python /usr/src/python/detect.py

cfg = EnvCfg()

def do_split(validation_split = 0.2, collapse_card_classes = False) :

    print(f"Cleaning {cfg.DETECT_DATA_PATH}")
    if cfg.DETECT_DATA_PATH.exists():
        shutil.rmtree(cfg.DETECT_DATA_PATH)

    print(f"Creating {cfg.DETECT_DATA_PATH}")
    cfg.DETECT_DATA_PATH.mkdir(parents=True, exist_ok=True)

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
    card_set = set(get_card_classes())
    orig_classes = read_classes()
    orig_class_map = get_class_map(orig_classes)

    # all 52 cards are now just 1 class called card
    new_classes = [c for c in orig_classes if c not in card_set]
    new_classes.append("Card")

    # also consolidate bet, call, and raise into one class "Chips in Pot"
    new_classes.remove("Bet")
    new_classes.remove("Call")
    new_classes.remove("Raise")

    new_classes.append("Chips in Pot")

    new_class_map = get_class_map(new_classes)
    card_index = new_class_map["Card"]
    cip_index = new_class_map["Chips in Pot"]

    # have old cards map back
    for c in card_set:
        new_class_map[c] = card_index

    # have old chips map back
    new_class_map["Bet"] = cip_index
    new_class_map["Call"] = cip_index
    new_class_map["Raise"] = cip_index

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

        if collapse_card_classes :
            replace_with_one_class(orig_classes, new_class_map, target_label_path / label_file.name)

    for f in train_files :
        shutil.copy(f, target_train_dir / cfg.IMAGE_FOLDER_NAME)
        label_file = (cfg.CARD_YOLO_PATH /  cfg.LABEL_FOLDER_NAME / f.stem).with_suffix(".txt")
        target_label_path = target_train_dir /  cfg.LABEL_FOLDER_NAME
        shutil.copy(label_file, target_label_path)

        if collapse_card_classes :
            replace_with_one_class(orig_classes, new_class_map, target_label_path / label_file.name)

    # lastly open cards_1.yml and replace the classes with the new ones
    with open(cfg.PYTHON_SRC_DIR / 'cards_1.yml', "r") as f:
        config = yaml.safe_load(f)

    config["names"] = {idx: name for idx, name in enumerate(new_classes)}

    with open(cfg.PYTHON_SRC_DIR / 'cards_1.yml', "w") as f:
        yaml.dump(config, f)

def replace_with_one_class(orig_classes: List[str], new_class_map: Dict[str, int], txt_file: Path):
    with open(txt_file, "r") as f:
        lines = f.readlines()
    
    with open(txt_file, "w") as f:
        for line in lines:
            fields = line.split(" ")
            orig_class = orig_classes[int(fields[0])]
            new_id = new_class_map[orig_class]
            fields[0] = str(new_id)
            f.write(" ".join(fields))



def train():
    
    
    # Load the model.
    model = YOLO(cfg.PYTHON_SRC_DIR / 'yolov8n.pt')
    
    # Training.
    results = model.train(
        data=cfg.PYTHON_SRC_DIR / 'cards_1.yml',
        imgsz=cfg.DETECT_IMG_SZ,
        epochs=200,
        batch=4,
        degrees=45,
        name=cfg.DETECT_MODEL_NAME)

def predict():

    model = YOLO(cfg.RUNS_DIR / 'detect' / cfg.DETECT_MODEL_NAME / 'weights/best.pt')

    predict_dir = cfg.PYTHON_SRC_DIR / "predictions"

    image_dir = cfg.DETECT_DATA_PATH / cfg.VALID_FOLDER_NAME / cfg.IMAGE_FOLDER_NAME

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
    cfg.DETECT_PROJECT_PATH.mkdir(parents=True, exist_ok=True)

    print(f"Cleaning {cfg.DETECT_PROJECT_PATH}")

    for sub_dir in cfg.DETECT_PROJECT_PATH.iterdir():
        if sub_dir.is_dir():
            rmtree(sub_dir)
        else:
            sub_dir.unlink()

if __name__ == "__main__":
    
    do_split(0.2, True)

    #sys.exit(0)
    clean_run_dir()

    train()    
    predict()