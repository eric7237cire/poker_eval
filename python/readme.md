# Screenshots 

Used Anaconda installation on windows

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


	
yolo task=detect mode=train model=yolov8n.pt imgsz=1280 data=pothole_v8.yaml epochs=50 batch=8 name=yolov8n_v8_50e


yolo task=detect mode=train model=yolov8n.pt imgsz=1280 data=zynga.yaml ep
ochs=50 batch=8 name=yolov8n_v8_50e

https://github.com/ultralytics/ultralytics/issues/2546

will downscale to the size

yolo task=detect mode=train model=yolov8n.pt imgsz=640 data=zynga.yaml epochs=100 name=yolov8n_v8_50e

yolo detect predict model=runs/detect/yolov8n_v8_50e6/weights/best.pt source='/home/eric/git/poker_eval/python/zynga_0.png'

yolo detect predict model=runs/detect/yolov8n_v8_50e/weights/best.pt source='/home/eric/git/poker_eval/python/datasets/zynga/train/images/0b057b90-zynga_0.png'

# Starting label studio

i:\python\Scripts\label-studio.exe start 

docker pull heartexlabs/label-studio:latest
docker run -it -p 9142:8080 -v /home/eric/git/poker_eval/data/label-studio:/label-studio/data heartexlabs/label-studio:latest

docker run -it --user root -v /home/eric/git/poker_eval/data/label-studio:/label-studio/data heartexlabs/label-studio:latest chown -R 1001:root /label-studio/data/

# Start jupyter

juyter lab

tensorboard --logdir runs  # replace with 'runs' directory

