# coding:utf-8
print(f"Importing...")

import json
from pathlib import Path
import shutil
import time
from typing import Dict, List, Tuple
import cv2
import random
from matplotlib.pyplot import box
from PIL import Image
import yaml
print(f"Importing Yolo...")
from ultralytics import YOLO
print(f"Done importing Yolo")
from env_cfg import EnvCfg
from classify import get_class_map, read_classes
import socket 
import io 
import numpy as np
cfg = EnvCfg()

print(f"Done importing")

def process_screenshots():
    """
    This processes the screenshots with QA in mind, saving the annotated screenshots
    """
    print(f"Loading detect model")
    detect_model = YOLO(cfg.RUNS_DIR / 'detect' / cfg.DETECT_MODEL_NAME / 'weights/best.pt')

    print(f"Loading classify model")
    classify_model = YOLO(cfg.CLASSIFY_PROJECT_PATH / cfg.CLASSIFY_MODEL_NAME / 'weights/best.pt')

    classes = read_classes()
    orig_classes = read_classes()
    orig_class_map = get_class_map(orig_classes)
    
    colors = [[random.randint(0, 255) for _ in range(3)]
              for _ in range(len(classes))]
    
    with open(cfg.PYTHON_SRC_DIR / 'cards_1.yml', "r") as f:
        config = yaml.safe_load(f)



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
            save=True
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

        # print(F"Box coords: {box_coords}\nSave dir: {save_dir}\nConfidence: {box_confidence_values}\nClasses: {box_classes}")   
        print(F"Save dir: {save_dir}\nConfidence: {box_confidence_values}\nClasses: {box_classes}")

        # Save the original image to the live yolo format            
        save_image_path.parent.mkdir(parents=True, exist_ok=True)
        shutil.copy(image_file, save_image_path)

        img = Image.open(image_file)
        run_classification(img, config["names"], orig_class_map, box_classes, box_coords, box_coords_normalized, classify_model, image_file.stem)
        
        # We also want to visualize the results
        annotated_image_path = cfg.LIVE_CARD_IMAGES_PATH / image_file.name
        print(f"Producing annotated image [{annotated_image_path}]")
        draw_box_on_image(image_file, classes, colors, annotated_image_path)

   


def run_classification(img: Image.Image, 
                       # mapping class index => class name (e.g.) 1: Check, see cards_1.yml
                       names: Dict[int, str],
                       # The yolo dataset we use to train has more classes, this maps the class name to the index in the yolo dataset
                       orig_class_map: Dict[str, int],
                       # predicted class index (see cards_1.yml)
                       box_classes: List[float],
                       # center x, center y, width, height
                       box_coords: List[List[float]], 
                       # center x, center y, width, height but between 0 and 1 normalized by total image width or height
                       box_coords_normalized, 
                       classify_model, save_stem: str) -> List[int]:
    """
    Saves each card image to a folder, and runs classification on each card image
    Also writes entries in yolo format to the LIVE dataset; meant to enhance training data
    """
    # Extract each image
   

    label_lines = []

    ret = []

    for box_index in range(0, len(box_coords)):

        predicted_class_index = int(box_classes[box_index])
        assert len(names) == 9
        predicted_class_name = names[predicted_class_index]

        print(f"For box #{box_index}, predicted class name: {predicted_class_name}")

        if predicted_class_name == "Card":
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

            if save_stem:
                card_image_path = cfg.LIVE_CARD_IMAGES_PATH / save_stem / f"{box_index}.png"
                
                card_image_path.parent.mkdir(parents=True, exist_ok=True)
                cropped_img.save(card_image_path)
            else:
                card_image_path = None

            # Convert PIL Image to NumPy array in BGR format
            cropped_image_np = np.array(cropped_img)[:, :, ::-1]

            classify_results = classify_model.predict(
                cropped_image_np, conf=0.25, 
                project=cfg.CLASSIFY_PROJECT_PATH,
                imgsz=cfg.CLASSIFY_IMG_SZ)
            
            classify_result = classify_results[0]

            classify_names = classify_result.names
            top_1_index = classify_result.probs.top1

            ret.append(top_1_index)

            top_1 = classify_names[top_1_index]

            orig_index = orig_class_map[top_1]
            
            if save_stem and card_image_path:
                txt_path = card_image_path.with_suffix(".txt").with_stem(f"{box_index}_{top_1}")
                txt_path.touch()

                label_lines.append(f"{orig_index} {box_coords_normalized[box_index][0]} {box_coords_normalized[box_index][1]} {box_coords_normalized[box_index][2]} {box_coords_normalized[box_index][3]}")
        else:
            # need to convert Chips in Pot => Bet
            if predicted_class_name == "Chips in Pot":
                predicted_class_name = "Bet"
            orig_index = orig_class_map[predicted_class_name]

            label_lines.append(f"{orig_index} {box_coords_normalized[box_index][0]} {box_coords_normalized[box_index][1]} {box_coords_normalized[box_index][2]} {box_coords_normalized[box_index][3]}")              

    if save_stem:
        save_label_path = (cfg.LIVE_PATH / cfg.LABEL_FOLDER_NAME / save_stem).with_suffix(".txt")
        save_label_path.parent.mkdir(parents=True, exist_ok=True)
        with open(save_label_path, "w") as f:
            if len(label_lines) > 0:
                f.write("\n".join(label_lines))
            else:
                f.write("")

    # else:
    #     # clean 
    #     dir_to_clean = cfg.LIVE_CARD_IMAGES_PATH / image_file.stem 
    #     if dir_to_clean.exists():
    #         shutil.rmtree( dir_to_clean )

    return ret

def clean_all():
    """
    Since all the images we want to process is in the incoming/save folder, clean out 
    the live , live_images
    """
    for sub_dir in (cfg.RUNS_DIR / "detect" ).iterdir():
        if sub_dir.is_dir() and sub_dir.name.startswith("predict"):

            shutil.rmtree(sub_dir)

    
    for path_to_clean in [cfg.LIVE_CARD_IMAGES_PATH, cfg.LIVE_PATH / cfg.IMAGE_FOLDER_NAME, cfg.LIVE_PATH / cfg.LABEL_FOLDER_NAME]:
        if path_to_clean.exists():
            shutil.rmtree(path_to_clean)
    
        path_to_clean.mkdir(parents=True, exist_ok=True)


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



def receive_image(sock):
    data = b''
    while True:
        packet = sock.recv(4096)
        if not packet: 
            break
        data += packet

    image_stream = io.BytesIO(data)
    image_stream.seek(0)
    image = Image.open(image_stream)
    return image


def process_screenshots_for_json():
    """
    This processes the screenshots only to get the cards, and write it to a json 
    """
    print(f"Loading detect model")
    detect_model = YOLO(cfg.RUNS_DIR / 'detect' / cfg.DETECT_MODEL_NAME / 'weights/best.pt')
    print(f"Loading classify model")
    classify_model = YOLO(cfg.CLASSIFY_PROJECT_PATH / cfg.CLASSIFY_MODEL_NAME / 'weights/best.pt')

    classes = read_classes()

    host = "host.docker.internal"
    port = 4242
    
    last_json_data = None
    
    while True:
        try:    
            print(f"Connecting to {host}:{port} with")
            with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as s:
                print(f"Connecting to {host}:{port}")
                s.connect((host, port))
                print(f"Sending a 1")
                s.sendall(b'1')
                print(f"Receiving image")   
                image = receive_image(s)
                print(f"Received image")
                # Convert PIL Image to NumPy array in BGR format
                image_np = np.array(image)[:, :, ::-1]

            try:
                results = detect_model.predict(
                    image_np, conf=0.55, 
                    imgsz=cfg.DETECT_IMG_SZ,
                    save=False
                )
            except Exception as e:
                print(f"Exception {e} in loading data {len(image_np)}, removing")
                
                time.sleep(2.5)
                continue

            result = results[0]

            # print(f"Predicted {image_file} to {result}")
            print(f"Predicted buffer")

            save_dir = result.save_dir

            # center x, center y, width, height
            box_coords = result.boxes.xywh.tolist()
            box_coords_normalized = result.boxes.xywhn.tolist()
            box_confidence_values = result.boxes.conf.tolist()
            box_classes = result.boxes.cls.tolist()

            print(F"Box coords: {box_coords}\nSave dir: {save_dir}\nConfidence: {box_confidence_values}\nClasses: {box_classes}")   

            results = run_classification(image, box_coords, box_coords_normalized, classify_model, False)
            
            result_classes = [classes[result] for result in results]

            # we want lowest ones first, then left to right, anything within pixel_chunk pixels
            pixel_chunk = 50
            int_box_coords: List[Tuple[int, int]] = [ (int(bc[0]/pixel_chunk), int(bc[1]/pixel_chunk)) for bc in box_coords]
            box_y_index = [ (-coords[1], coords[0], box_index) for (box_index, coords) in enumerate(int_box_coords) ]            
            box_y_index.sort()

            if not len(box_y_index) in [2, 5, 6, 7]:
                print(f"Skipping, strange number of cards: {len(box_y_index)}")
                
                time.sleep(0.70)
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

            json_data = {
                "hole_cards": f"{hole_card_class1} {hole_card_class2}",
                "board_cards": " ".join(board_classes)
            }
            if json_data != last_json_data:
                print(f"Writing json to {json_data}")
                with open(cfg.LIVE_JSON_PATH, "w") as f:
                    json.dump(json_data, f)

                last_json_data = json_data
            else:
                print("Skipping, no change")

            time.sleep(0.75)
        except Exception as e:
            print("Exception!")
            print(e)




        
if __name__ == '__main__':
    print(f"Clean detect folder")
    clean_all()
    # saves in annotated
    process_screenshots()

    # process_screenshots_for_json()
