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

In WSL

```
python3 -m pip install torch torchvision
python3 -m pip install ultralytics
```

nvidia-smi

Install cuda

https://developer.nvidia.com/cuda-downloads?target_os=Linux&target_arch=x86_64&Distribution=WSL-Ubuntu&target_version=2.0&target_type=deb_local

Easier likely is using ultralytics docker container with nvidia docker driver


# Running ultralytics in docker

cd git/poker_eval
source ./dev/local.env

docker run -it --rm  \
--ipc=host --gpus all \
-p 6006:6006 \
-v ${REPO_ROOT}/python:/usr/src/python \
-v ${REPO_ROOT}/python/datasets:/usr/src/datasets \
-v ${REPO_ROOT}/python/runs:/usr/src/ultralytics/runs \
-v ${REPO_ROOT}/vue-poker/src/assets:/usr/src/assets \
ultralytics/ultralytics:latest

pip install pydantic.settings
pip install torchsummary
pip install -U ultralytics

## Tensorboard 

Run before training
tensorboard --logdir /usr/src/ultralytics/runs --bind_all & 

## Training card detector using yolo

python /usr/src/python/detect.py

check bottom for which functions are being called

All params in /usr/src/ultralytics/ultralytics/cfg/default.yaml (open in vscode to running container)


# Starting label studio

docker pull heartexlabs/label-studio:latest
docker run -d -it -p 9142:8080 \
-e LOCAL_FILES_SERVING_ENABLED=true \
-v ${REPO_ROOT}/data/label-studio:/label-studio/data \
-v ${REPO_ROOT}/python:/home/user/python-data \
heartexlabs/label-studio:latest

## Converting from yolo

Improvement loop, use get/process screenshot to get more data
which creates yolo dataset in live

label studio exports bbox format, so
run convert_to_yolo.py

mkdir ${REPO_ROOT}/python/label_studio_import
chmod 777 -R ${REPO_ROOT}/python/label_studio_import

cp ${REPO_ROOT}/python/datasets/all/classes.txt ${REPO_ROOT}/python/datasets/live/classes.txt 
cp ${REPO_ROOT}/python/datasets/all/notes.json ${REPO_ROOT}/python/datasets/live/notes.json


export DATASET_NAME=live

Use all images link in the url, because that's where they will stay

docker run -it --rm \
-v ${REPO_ROOT}/python:/poker/data \
-v ${REPO_ROOT}/python:/python-data \
heartexlabs/label-studio:latest \
label-studio-converter import yolo \
-i /poker/data/datasets/${DATASET_NAME} \
-o /poker/data/label_studio_import/${DATASET_NAME}.json \
--image-ext .png --out-type annotations \
--image-root-url /data/local-files/?d=/home/user/python-data/datasets/all/images/

Import live.json from 
\\wsl.localhost\Ubuntu-22.04\home\eric\git\poker_eval\python\label_studio_import

Fix stuff

(keep old all directory because the image links use that)

Reexport to all / unzip

Retrain

## To fix dir permissions

source ./dev/local.env
docker run -it --rm --user root -v ${REPO_ROOT}/data/label-studio:/label-studio/data heartexlabs/label-studio:latest chown -R 1001:root /label-studio/data/

## Adding local storage

Storage Type: Local Files
Absolute Local path: /home/user/python-data  (in /home/user)

# Start jupyter

juyter lab
source ./dev/local.env
docker run --rm --detach -p 8888:8888  -v ${REPO_ROOT}/python:/home/jovyan/work quay.io/jupyter/pytorch-notebook:latest

In visual studio code, need to copy the url with the token and sometimes the token for it to connect

#  start-notebook.sh --ip='*' --NotebookApp.token='' --NotebookApp.password=''

Using in visual studio code

# Card classfier 

## Links

https://pytorch.org/tutorials/beginner/transfer_learning_tutorial.html
https://github.com/dickreuter/Poker/blob/master/poker/scraper/table_scraper_nn.py