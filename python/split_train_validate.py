from pathlib import Path
import random
import shutil

# BASE_DIR = Path("/home/eric/git/poker_eval/python")

BASE_DIR = Path("/usr/src")

# this is where the yolo export from label studio was unzipped, it should contain an images and labels folder
YOLO_EXPORT_DIR = BASE_DIR / "datasets" / "all"

TARGET_DIR = BASE_DIR / "datasets" / "zynga"

IMAGES_DIR_NAME = "images"
LABELS_DIR_NAME = "labels"

TARGET_TRAIN_DIR = TARGET_DIR / "train"
TARGET_VALIDATE_DIR = TARGET_DIR / "valid"

def do_split(validation_split = 0.2, collapse_to_one_class = False) :

    shutil.rmtree(TARGET_DIR)

    image_files = [f for f in (YOLO_EXPORT_DIR / IMAGES_DIR_NAME).iterdir()]
    label_files = [f for f in (YOLO_EXPORT_DIR / LABELS_DIR_NAME).iterdir()]
    
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

    for dir_base in [TARGET_TRAIN_DIR, TARGET_VALIDATE_DIR] :
        for dir_name in [IMAGES_DIR_NAME, LABELS_DIR_NAME] :
            (dir_base / dir_name).mkdir(parents=True, exist_ok=True)
    
    # copy files
    for f in validate_files :
        shutil.copy(f, TARGET_VALIDATE_DIR / IMAGES_DIR_NAME)
        label_file = (YOLO_EXPORT_DIR / LABELS_DIR_NAME / f.stem).with_suffix(".txt")
        target_label_path = TARGET_VALIDATE_DIR / LABELS_DIR_NAME
        shutil.copy(label_file, target_label_path)

        if collapse_to_one_class :
            replace_with_one_class(target_label_path / label_file.name)

    for f in train_files :
        shutil.copy(f, TARGET_TRAIN_DIR / IMAGES_DIR_NAME)
        label_file = (YOLO_EXPORT_DIR / LABELS_DIR_NAME / f.stem).with_suffix(".txt")
        target_label_path = TARGET_TRAIN_DIR / LABELS_DIR_NAME
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

if __name__ == '__main__' :
    do_split(0.20, True)