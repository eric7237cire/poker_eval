# It seems label-studio exports in bbox style format
from pathlib import Path
import shutil
from env_cfg import EnvCfg
from PIL import Image

cfg = EnvCfg(PYTHON_SRC_DIR=Path("/home/eric/git/poker_eval/python"))

def convert_to_yolo():

    for file_name in ["classes.txt", "notes.json"]:
        shutil.copyfile(cfg.CARD_YOLO_PATH / file_name , 
                    cfg.YOLO_CORRECTED_PATH / file_name)
        
    shutil.copytree(cfg.CARD_YOLO_PATH / cfg.IMAGE_FOLDER_NAME, cfg.YOLO_CORRECTED_PATH / cfg.IMAGE_FOLDER_NAME, dirs_exist_ok=True)

    for label_filename in (cfg.CARD_YOLO_PATH / cfg.LABEL_FOLDER_NAME).iterdir():

        image_filename = (cfg.YOLO_CORRECTED_PATH / cfg.IMAGE_FOLDER_NAME / label_filename.stem).with_suffix(".png")

        img = Image.open(image_filename)

        with open(label_filename, "r") as f:
            lines = f.readlines()

        target_label_filename = (cfg.YOLO_CORRECTED_PATH / cfg.LABEL_FOLDER_NAME / label_filename.name)
        target_label_filename.parent.mkdir(parents=True, exist_ok=True)
        with open(target_label_filename, "w") as f:
            for line in lines:
                fields = line.split(" ")
                class_index = fields[0]
                xs = []
                ys = []
                for i in range(0, 4):
                    xs.append(float(fields[1 + i * 2]))
                    ys.append(float(fields[2 + i * 2]))
                
                x_min = min(xs) 
                x_max = max(xs)
                y_min = min(ys)
                y_max = max(ys)

                # fields are already normalized to 0-1

                center_x = (x_min + x_max) / 2
                center_y = (y_min + y_max) / 2
                width = x_max - x_min
                height = y_max - y_min

                f.write(f"{class_index} {center_x} {center_y} {width} {height}\n")

if __name__ == "__main__":
    convert_to_yolo()    