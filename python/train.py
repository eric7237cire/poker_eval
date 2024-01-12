from shutil import rmtree
import shutil
from pathlib import Path
from ultralytics import YOLO

def train():
    

    
    # Load the model.
    model = YOLO('yolov8n.pt')
    
    # Training.
    results = model.train(
        data='zynga_1.yml',
        imgsz=640,
        epochs=10,
        batch=4,
        name='yolo1')

def predict():

    model = YOLO('/usr/src/ultralytics/runs/detect/yolo16/weights/best.pt')

    predict_dir = Path("/eric/python/predictions")

    image_dir = Path("/usr/src/datasets/zynga/train/images/")

    if predict_dir.exists():
        shutil.rmtree(predict_dir)

    predict_dir.mkdir(parents=True, exist_ok=True)

    for image_file in image_dir.iterdir():
        if image_file.suffix != ".png":
            continue

        target_path = predict_dir / image_file.name
        shutil.copy(image_file, target_path)

        print(f"Predicting {image_file} to {target_path}")
        model.predict(target_path, conf=0.05, save=True, imgsz=640)



if __name__ == "__main__":
    # train()    
    predict()