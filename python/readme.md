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

# Example command lines

From within ultralytics container
	
yolo task=detect mode=train model=yolov8n.pt imgsz=1280 data=pothole_v8.yaml epochs=50 batch=8 name=yolov8n_v8_50e


yolo task=detect mode=train model=yolov8n.pt imgsz=1280 data=zynga.yaml ep
ochs=50 batch=8 name=yolov8n_v8_50e

https://github.com/ultralytics/ultralytics/issues/2546

will downscale to the size

yolo task=detect mode=train model=yolov8n.pt imgsz=640 data=zynga.yaml epochs=100 name=yolov8n_v8_50e

yolo detect predict model=runs/detect/yolov8n_v8_50e6/weights/best.pt source='/home/eric/git/poker_eval/python/zynga_0.png'

yolo detect predict model=runs/detect/yolov8n_v8_50e/weights/best.pt source='/home/eric/git/poker_eval/python/datasets/zynga/train/images/0b057b90-zynga_0.png'

yolo detect predict model=/usr/src/ultralytics/runs/detect/yolo16/weights/best.pt source='/usr/src/datasets/zynga/valid/images/e70cd165-zynga_1.png'

yolo detect predict model=/usr/src/ultralytics/runs/detect/yolo16/weights/best.pt source='/usr/src/datasets/zynga/valid/images/94164c06-zynga_2.png'
 
 yolo detect predict model=/usr/src/ultralytics/runs/detect/yolo16/weights/best.pt source='/usr/src/datasets/zynga/train/images/8cc33bd4-zynga_1.png'


# Running ultralytics in docker

docker run -it --rm --ipc=host --gpus all -p 6006:6006 -v /home/eric/git/poker_eval/python:/usr/src/python -v /home/eric/git/poker_eval/python/datasets:/usr/src/datasets -v /home/eric/git/poker_eval/python/runs:/usr/src/ultralytics/runs ultralytics/ultralytics:latest

pip install pydantic.settings
pip install -U ultralytics

## Tensorboard 

Run before training
tensorboard --logdir /usr/src/ultralytics/runs --bind_all & 

## Training card detector using yolo

python /usr/src/python/detect.py

check bottom for which functions are being called

Using zynga_1.yml

All params in /usr/src/ultralytics/ultralytics/cfg/default.yaml (open in vscode to running container)


# Starting label studio

docker pull heartexlabs/label-studio:latest
docker run -it -p 9142:8080 -v /home/eric/git/poker_eval/data/label-studio:/label-studio/data heartexlabs/label-studio:latest

## To fix dir permissions
docker run -it --user root -v /home/eric/git/poker_eval/data/label-studio:/label-studio/data heartexlabs/label-studio:latest chown -R 1001:root /label-studio/data/

# Start jupyter

juyter lab

docker run --rm -p 8888:8888  -v /home/eric/git/poker_eval/python:/home/jovyan/work quay.io/jupyter/pytorch-notebook:latest


# Card classfier 

## Links

https://pytorch.org/tutorials/beginner/transfer_learning_tutorial.html
https://github.com/dickreuter/Poker/blob/master/poker/scraper/table_scraper_nn.py