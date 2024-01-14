import json
from pathlib import Path
import shutil
import time
from typing import List, Tuple
# coding:utf-8
import cv2
import os
import random
from matplotlib.pyplot import box
from PIL import Image
from ultralytics import YOLO
from env_cfg import EnvCfg
from classify import read_classes

cfg = EnvCfg()

def process_screenshots():
    """
    This processes the screenshots with QA in mind, saving the annotated screenshots
    """
    
    detect_model = YOLO(cfg.RUNS_DIR / 'detect' / cfg.DETECT_MODEL_NAME / 'weights/best.pt')
    classify_model = YOLO(cfg.CLASSIFY_PROJECT_PATH / cfg.CLASSIFY_MODEL_NAME / 'weights/best.pt')

    classes = read_classes()
    colors = [[random.randint(0, 255) for _ in range(3)]
              for _ in range(len(classes))]

    for i in range(0, 10_000):
    
        images = cfg.INCOMING_PATH.glob("*.png")

        for image_file in images:
            # Run detection on the screenshot
            save_image_path = cfg.LIVE_PATH / cfg.IMAGE_FOLDER_NAME / image_file.name

            if save_image_path.exists():
                print(f"Skipping {image_file}, already processed")
                continue

            results = detect_model.predict(
                image_file, conf=0.25, 
                imgsz=cfg.DETECT_IMG_SZ,
                save=False
            )
            result = results[0]

            # print(f"Predicted {image_file} to {result}")
            print(f"Predicted {image_file}")

            save_dir = result.save_dir

            # center x, center y, width, height
            box_coords = result.boxes.xywh.tolist()
            box_coords_normalized = result.boxes.xywhn.tolist()
            box_confidence_values = result.boxes.conf.tolist()
            box_classes = result.boxes.cls.tolist()

            print(F"Box coords: {box_coords}\nSave dir: {save_dir}\nConfidence: {box_confidence_values}\nClasses: {box_classes}")   

            # Save the original image to the live yolo format            
            save_image_path.parent.mkdir(parents=True, exist_ok=True)
            shutil.copy(image_file, save_image_path)

            run_classification(image_file, box_coords, box_coords_normalized, classify_model, True)
            
            # We also want to visualize the results
            annotated_image_path = cfg.LIVE_CARD_IMAGES_PATH / image_file.name
            print(f"Producing annotated image [{annotated_image_path}]")
            draw_box_on_image(image_file, classes, colors, annotated_image_path)

        

        # Sleep 500 ms
        time.sleep(2.5)

        # break


def run_classification(image_file: Path, box_coords: List[List[float]], box_coords_normalized, classify_model, save: bool) -> List[int]:
    """
    Saves each card image to a folder, and runs classification on each card image
    Also writes entries in yolo format to the LIVE dataset; meant to enhance training data
    """
    # Extract each image
    img = Image.open(image_file)

    label_lines = []

    ret = []

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

        ret.append(top_1_index)

        top_1 = names[top_1_index]
        
        if save:
            txt_path = card_image_path.with_suffix(".txt").with_stem(f"{box_index}_{top_1}")
            txt_path.touch()

            label_lines.append(f"{top_1_index} {box_coords_normalized[box_index][0]} {box_coords_normalized[box_index][1]} {box_coords_normalized[box_index][2]} {box_coords_normalized[box_index][3]}")

    if save:
        save_label_path = (cfg.LIVE_PATH / cfg.LABEL_FOLDER_NAME / image_file.name).with_suffix(".txt")
        save_label_path.parent.mkdir(parents=True, exist_ok=True)
        with open(save_label_path, "w") as f:
            if len(label_lines) > 0:
                f.write("\n".join(label_lines))
            else:
                f.write("")

    else:
        # clean 
        dir_to_clean = cfg.LIVE_CARD_IMAGES_PATH / image_file.stem 
        if dir_to_clean.exists():
            shutil.rmtree( dir_to_clean )

    return ret

def clean_detect():
    for sub_dir in (cfg.RUNS_DIR / "detect" ).iterdir():
        if sub_dir.is_dir() and sub_dir.name.startswith("predict"):

            shutil.rmtree(sub_dir)

    if cfg.LIVE_CARD_IMAGES_PATH.exists():
        shutil.rmtree(cfg.LIVE_CARD_IMAGES_PATH)
    cfg.LIVE_CARD_IMAGES_PATH.mkdir(parents=True, exist_ok=True)


# https://github.com/waittim/draw-YOLO-box/blob/main/draw_box.py


def plot_one_box(x, image, color=None, label=None, line_thickness=None):
    # Plots one bounding box on image img
    tl = line_thickness or round(
        0.002 * (image.shape[0] + image.shape[1]) / 2) + 1  # line/font thickness
    color = color or [random.randint(0, 255) for _ in range(3)]
    c1, c2 = (int(x[0]), int(x[1])), (int(x[2]), int(x[3]))
    cv2.rectangle(image, c1, c2, color, thickness=tl, lineType=cv2.LINE_AA)
    if label:
        tf = max(tl - 1, 1)  # font thickness
        t_size = cv2.getTextSize(label, 0, fontScale=tl / 3, thickness=tf)[0]
        c2 = c1[0] + t_size[0], c1[1] - t_size[1] - 3
        cv2.rectangle(image, c1, c2, color, -1, cv2.LINE_AA)  # filled
        cv2.putText(image, label, (c1[0], c1[1] - 2), 0, tl / 3,
                    [225, 255, 255], thickness=tf, lineType=cv2.LINE_AA)

def draw_box_on_image(image_path: Path, classes: List[str], colors, output_path: Path):
    """
    This function will add rectangle boxes on the images.
    """
    
    label_path = (cfg.LIVE_PATH / cfg.LABEL_FOLDER_NAME / image_path.name).with_suffix(".txt")
   
    with open(label_path, 'r') as f:
        lines = f.readlines()
   
    image = cv2.imread(str(image_path))
    try:
        height, width, channels = image.shape
    except:
        print('no shape info.')
        return 0

    for line in lines:
        split_lines = line.split()
        class_idx = int(split_lines[0])

        x_center, y_center, w, h = float(
            split_lines[1])*width, float(split_lines[2])*height, float(split_lines[3])*width, float(split_lines[4])*height
        x1 = round(x_center-w/2)
        y1 = round(y_center-h/2)
        x2 = round(x_center+w/2)
        y2 = round(y_center+h/2)

        plot_one_box([x1, y1, x2, y2], image, color=colors[class_idx],
                     label=classes[class_idx], line_thickness=None)

        cv2.imwrite(str(output_path), image)


def process_screenshots_for_json():
    """
    This processes the screenshots only to get the cards, and write it to a json 
    """
    
    detect_model = YOLO(cfg.RUNS_DIR / 'detect' / cfg.DETECT_MODEL_NAME / 'weights/best.pt')
    classify_model = YOLO(cfg.CLASSIFY_PROJECT_PATH / cfg.CLASSIFY_MODEL_NAME / 'weights/best.pt')

    classes = read_classes()
    
    
    for i in range(0, 10_000):
        try:    
            images = list(cfg.INCOMING_PATH.glob("*.png"))

            if len(images) == 0:
                print("No images found")
                time.sleep(0.5)
                continue

            most_recent = max(images, key=lambda p: p.stat().st_ctime)

            print(f"Processing {most_recent}")

            image_file = most_recent

            try:
                results = detect_model.predict(
                    image_file, conf=0.55, 
                    imgsz=cfg.DETECT_IMG_SZ,
                    save=False
                )
            except Exception as e:
                print(f"Exception {e} in loading file {image_file}, removing")
                image_file.unlink()

                time.sleep(0.5)
                continue

            result = results[0]

            # print(f"Predicted {image_file} to {result}")
            print(f"Predicted {image_file}")

            save_dir = result.save_dir

            # center x, center y, width, height
            box_coords = result.boxes.xywh.tolist()
            box_coords_normalized = result.boxes.xywhn.tolist()
            box_confidence_values = result.boxes.conf.tolist()
            box_classes = result.boxes.cls.tolist()

            print(F"Box coords: {box_coords}\nSave dir: {save_dir}\nConfidence: {box_confidence_values}\nClasses: {box_classes}")   

            results = run_classification(image_file, box_coords, box_coords_normalized, classify_model, False)
            
            result_classes = [classes[result] for result in results]

            # we want lowest ones first, then left to right, anything within pixel_chunk pixels
            pixel_chunk = 50
            int_box_coords: List[Tuple[int, int]] = [ (int(bc[0]/pixel_chunk), int(bc[1]/pixel_chunk)) for bc in box_coords]
            box_y_index = [ (-coords[1], coords[0], box_index) for (box_index, coords) in enumerate(int_box_coords) ]            
            box_y_index.sort()

            if not len(box_y_index) in [2, 5, 6, 7]:
                print(f"Skipping {image_file}, strange number of cards: {len(box_y_index)}")
                image_file.unlink()
                time.sleep(0.25)
                continue

            hole_card_index1 = box_y_index[0][2]
            hole_card_class1 = result_classes[hole_card_index1]
            hole_card_index2 = box_y_index[1][2]
            hole_card_class2 = result_classes[hole_card_index2]

            
            board_card_indexes = [box_y_index[i][2] for i in range(2, len(box_y_index))]
            
            board_classes = [result_classes[i] for i in board_card_indexes]
            print(f"Result classes: {result_classes}")
            print(f"Board classes: {board_classes}")
            print(f"Hole cards: {hole_card_class1}, {hole_card_class2}")

            with open(cfg.LIVE_JSON_PATH, "w") as f:
                json.dump({
                    "hole_cards": f"{hole_card_class1} {hole_card_class2}",
                    "board_cards": " ".join(board_classes)
                }, f)

            print(f"Deleting {image_file}")
            image_file.unlink()
            time.sleep(0.25)
        except Exception as e:
            print("Exception!")
            print(e)




        
if __name__ == '__main__':
    clean_detect()
    # saves in annotated
    # process_screenshots()

    process_screenshots_for_json()
