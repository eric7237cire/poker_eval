# Screenshots 

Used Anaconda installation on windows

```
I:\Miniconda3\python.exe --version


d:\anaconda3\Scripts\pip.exe install pygetwindow
pyautogui
d:\anaconda3\python.exe "\\wsl.localhost\Ubuntu-20.04\home\eric\git\poker_eval\python\get_screenshot.py"
```

# ML setup

https://learnopencv.com/train-yolov8-on-custom-dataset/


nvidia-smi

Install cuda

https://developer.nvidia.com/cuda-downloads?target_os=Linux&target_arch=x86_64&Distribution=WSL-Ubuntu&target_version=2.0&target_type=deb_local


# Running ultralytics in docker

cd git/poker_eval/python
source ../dev/local.env

docker-compose run --rm ultralytics_service

## Tensorboard 

Run before training
tensorboard --logdir /usr/src/ultralytics/runs --bind_all & 

## Training card detector using yolo

python /usr/src/python/detect.py

check bottom for which functions are being called

All params in /usr/src/ultralytics/ultralytics/cfg/default.yaml (open in vscode to running container)


# Starting label studio

docker-compose up -d label_studio_service

## Getting more training data

Improvement loop, use get/process screenshot to get more data
which creates yolo dataset in live

$REPO_ROOT/dev/save_ss.bat

This saves each image to $REPO_ROOT\python\datasets\incoming\save

Run in ultralytics container process_screenshot,
call `process_screenshots` not `process_screenshots_for_json`

label studio exports bbox format, so
run convert_to_yolo.py following instructions below

### Importing live

1. Import dir creation

mkdir ${REPO_ROOT}/python/label_studio_import
chmod 777 -R ${REPO_ROOT}/python/label_studio_import

This is where the json goes

2.  Classes & Notes 
Live dir should already exist from process_screenshots, which created labels.txt and placed images in yolo format

cp ${REPO_ROOT}/python/datasets/all/classes.txt ${REPO_ROOT}/python/datasets/live/classes.txt 
cp ${REPO_ROOT}/python/datasets/all/notes.json ${REPO_ROOT}/python/datasets/live/notes.json

3.  Create import json

be in python dir

. ../dev/local.env
export DATASET_NAME=live

Use all images link in the url, because that's where they will stay

For import to all
```
docker-compose run --rm label_studio_service  \
label-studio-converter import yolo \
-i /home/user/python-data/datasets/${DATASET_NAME} \
-o /home/user/python-data/label_studio_import/${DATASET_NAME}.json \
--image-ext .png --out-type annotations \
--image-root-url /data/local-files/?d=/home/user/python-data/datasets/all/images/
```

Copy images to all

Import to all dataset

Fix stuff

(keep old all directory because the image links use that)

Reexport to all / unzip

Retrain

####
For just looking (image link is live)
export DATASET_NAME=live
```
docker-compose run --rm label_studio_service  \
label-studio-converter import yolo \
-i /home/user/python-data/datasets/${DATASET_NAME} \
-o /home/user/python-data/label_studio_import/${DATASET_NAME}.json \
--image-ext .png --out-type annotations \
--image-root-url /data/local-files/?d=/home/user/python-data/datasets/live/images/
```

Create new project

Import live.json from 
\\wsl.localhost\Ubuntu-22.04\home\eric\git\poker_eval\python\label_studio_import

Copy paste the xml live.label_config.xml into the view settings in the ui

## Importing to label studio from scratch

Unzip to datasets/all

```
cd python

. ../dev/local.env
export DATASET_NAME=all

docker-compose run --rm label_studio_service  \
label-studio-converter import yolo \
-i /home/user/python-data/datasets/${DATASET_NAME} \
-o /home/user/python-data/label_studio_import/${DATASET_NAME}.json \
--image-ext .png --out-type annotations \
--image-root-url /data/local-files/?d=/home/user/python-data/datasets/all/images/

docker-compose up -d label_studio_service

Import
Copy xml from label_config.xml to view settings in ui

```
```

## To fix dir permissions

source ./dev/local.env
docker run -it --rm --user root -v ${REPO_ROOT}/data/label-studio:/label-studio/data heartexlabs/label-studio:latest chown -R 1001:root /label-studio/data/

## Adding local storage

Storage Type: Local Files
Absolute Local path: /home/user/python-data  (in /home/user)

# Start jupyter

cd python
source ../dev/local.env
docker-compose run --rm --detach jupyter_pytorch_service

In visual studio code, need to copy the url with the token and sometimes the token for it to connect



#  start-notebook.sh --ip='*' --NotebookApp.token='' --NotebookApp.password=''

Using in visual studio code

# Card classfier 

## Links

https://pytorch.org/tutorials/beginner/transfer_learning_tutorial.html
https://github.com/dickreuter/Poker/blob/master/poker/scraper/table_scraper_nn.py