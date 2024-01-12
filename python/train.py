from re import sub
from shutil import rmtree
import shutil
from pathlib import Path
from ultralytics import YOLO

# Runs in the ultralytics container
# switch function in main

RUNS_DIR = Path("/usr/src/ultralytics/runs")
PYTHON_SRC_DIR = Path("/eric/python")

# python /eric/python/train.py
def train():
    
    
    # Load the model.
    model = YOLO(PYTHON_SRC_DIR / 'yolov8n.pt')
    
    # Training.
    results = model.train(
        data=PYTHON_SRC_DIR / 'zynga_1.yml',
        imgsz=640,
        epochs=100,
        batch=4,
        name='yolo')

def predict():

    model = YOLO(RUNS_DIR / 'detect/yolo1/weights/best.pt')

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
        model.predict(target_path, conf=0.05, save=True, imgsz=640)


def clean_run_dir():
    for sub_dir in RUNS_DIR.iterdir():
        if sub_dir.is_dir():
            rmtree(sub_dir)
        else:
            sub_dir.unlink()

if __name__ == "__main__":
    # clean_run_dir()
    # train()    
    predict()