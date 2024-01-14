from pathlib import Path
from typing import List, Set
from env_cfg import EnvCfg

cfg = EnvCfg()

def prepare_import():
    # find anything that was deleted in annotated, and remove the corresponding entries in the live dataset
    pass

def remove_files(files: List[Path], annotated_images: Set[str]):
    for f in files:
        if f.stem in annotated_images:
            print(f"Has annotated, not deleting {f}")
            continue
        else:
            print(f"Removing {f}")
            f.unlink()


def clean_all():
    label_files = [f for f in (cfg.LIVE_PATH / cfg.LABEL_FOLDER_NAME).iterdir()]
    image_files = [f for f in (cfg.LIVE_PATH / cfg.IMAGE_FOLDER_NAME).iterdir()]
    incoming_files = [f for f in (cfg.INCOMING_PATH).iterdir()]

    annotated_stems = set([f.stem for f in cfg.LIVE_CARD_IMAGES_PATH.glob("*.png")])

    remove_files(label_files, annotated_stems)
    remove_files(image_files, annotated_stems)
    remove_files(incoming_files, annotated_stems)


if __name__ == "__main__":
    
    clean_all()
    # prepare_import()

    # run the following in WSL
    # the images will permanently live in all
    """
docker run -it --rm \
-v ${REPO_ROOT}/python:/poker/data \
-v ${REPO_ROOT}/python:/python-data \
heartexlabs/label-studio:latest \
label-studio-converter import yolo \
-i /poker/data/datasets/${DATASET_NAME} \
-o /poker/data/label_studio_import/${DATASET_NAME}.json \
--image-ext .png --out-type annotations \
--image-root-url /data/local-files/?d=/home/user/python-data/datasets/all/images/

    """

    # Now copy the images only to all

    # \\wsl.localhost\Ubuntu-22.04\home\eric\git\poker_eval\python\datasets\live\images to
    # \\wsl.localhost\Ubuntu-22.04\home\eric\git\poker_eval\python\datasets\all\images

    # Correct it

    # re-export it to all2