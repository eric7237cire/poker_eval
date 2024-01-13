import shutil
import time

from matplotlib.pyplot import box
from PIL import Image
from ultralytics import YOLO
from env_cfg import EnvCfg

cfg = EnvCfg()

def process_screenshots():
    
    detect_model = YOLO(cfg.RUNS_DIR / 'detect' / cfg.DETECT_MODEL_NAME / 'weights/best.pt')
    classify_model = YOLO(cfg.CLASSIFY_PROJECT_PATH / cfg.CLASSIFY_MODEL_NAME / 'weights/best.pt')

    for i in range(0, 10_000):
    
        images = cfg.INCOMING_PATH.glob("*.png")

        for image_file in images:
            results = detect_model.predict(
                image_file, conf=0.25, 
                imgsz=cfg.DETECT_IMG_SZ,
                save=True
            )
            result = results[0]

            print(f"Predicted {image_file} to {result}")

            save_dir = result.save_dir

            # center x, center y, width, height
            box_coords = result.boxes.xywh.tolist()
            box_coords_normalized = result.boxes.xywhn.tolist()
            box_confidence_values = result.boxes.conf.tolist()
            box_classes = result.boxes.cls.tolist()

            print(F"Box coords: {box_coords}\nSave dir: {save_dir}\nConfidence: {box_confidence_values}\nClasses: {box_classes}")   

            # Save the original image
            save_image_path = cfg.LIVE_PATH / cfg.IMAGE_FOLDER_NAME / image_file.name
            save_image_path.parent.mkdir(parents=True, exist_ok=True)
            shutil.copy(image_file, save_image_path)

            # Extract each image
            img = Image.open(image_file)

            label_lines = []

            for box_index in range(0, len(box_coords)):
                center_x, center_y, width, height = box_coords[box_index]
                x_min = center_x - width / 2
                x_max = center_x + width / 2
                y_min = center_y - height / 2
                y_max = center_y + height / 2

                # open the png image and crop this box out
                # Crop the image to the specified rectangle

                cropped_img = img.crop((x_min, y_min, x_max, y_max))

                # Resize the image 
                cropped_img = cropped_img.resize((cfg.CLASSIFY_IMG_SZ, cfg.CLASSIFY_IMG_SZ))

                card_image_path = cfg.LIVE_CARD_IMAGES_PATH / image_file.stem / f"{box_index}.png"
                
                card_image_path.parent.mkdir(parents=True, exist_ok=True)
                cropped_img.save(card_image_path)

                classify_results = classify_model.predict(
                    card_image_path, conf=0.15, 
                    project=cfg.CLASSIFY_PROJECT_PATH,
                    imgsz=cfg.CLASSIFY_IMG_SZ)
                
                classify_result = classify_results[0]

                names = classify_result.names
                top_1_index = classify_result.probs.top1
                top_1 = names[top_1_index]
                
                txt_path = card_image_path.with_suffix(".txt").with_stem(f"{box_index}_{top_1}")
                txt_path.touch()

                label_lines.append(f"{top_1_index} {box_coords_normalized[box_index][0]} {box_coords_normalized[box_index][1]} {box_coords_normalized[box_index][2]} {box_coords_normalized[box_index][3]}")

            save_label_path = (cfg.LIVE_PATH / cfg.LABEL_FOLDER_NAME / image_file.name).with_suffix(".txt")
            save_label_path.parent.mkdir(parents=True, exist_ok=True)
            with open(save_label_path, "w") as f:
                f.write("\n".join(label_lines))

        # Sleep 500 ms
        time.sleep(0.5)

        break

def clean_detect():
    for sub_dir in (cfg.RUNS_DIR / "detect" ).iterdir():
        if sub_dir.is_dir() and sub_dir.name.startswith("predict"):

            shutil.rmtree(sub_dir)

if __name__ == '__main__':
    clean_detect()
    process_screenshots()
