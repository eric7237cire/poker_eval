from re import sub
from shutil import rmtree
import shutil
from pathlib import Path
from ultralytics import YOLO

# Runs in the ultralytics container
# switch function in main
# Runs 'detect' finding the cards in the screenshot

# Run split_train_validate 1st, which also collapses the classes into one class, a card

RUNS_DIR = Path("/usr/src/ultralytics/runs")
PYTHON_SRC_DIR = Path("/eric/python")

# we can use whatever name
MODEL_NAME="yolo"

def train():
    
    
    # Load a model
    model = YOLO('yolov8n-cls.pt')  # load a pretrained model (recommended for training)

    # Train the model
    results = model.train(data='path/to/dataset', epochs=100, imgsz=640)
    
    # Training.
    results = model.train(
        data=PYTHON_SRC_DIR / 'zynga_1.yml',
        imgsz=640,
        epochs=100,
        batch=4,
        name=MODEL_NAME)

def predict():

    model = YOLO(RUNS_DIR / 'detect' / MODEL_NAME / 'weights/best.pt')

    predict_dir = PYTHON_SRC_DIR / "predictions"

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
        model.predict(target_path, conf=0.15, save=True, imgsz=640)

    shutil.rmtree(predict_dir)

def clean_run_dir():
    for sub_dir in RUNS_DIR.iterdir():
        if sub_dir.is_dir():
            rmtree(sub_dir)
        else:
            sub_dir.unlink()

if __name__ == "__main__":
    # clean_run_dir()
    # train()    
    # predict()