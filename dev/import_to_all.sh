sh_dir=`dirname $0`
source ${sh_dir}/../dev/local.env
echo $REPO_ROOT

DATASET_NAME="live"

echo $DATASET_NAME

cd $REPO_ROOT/python

docker-compose run --rm label_studio_service  \
label-studio-converter import yolo \
-i /home/user/python-data/datasets/${DATASET_NAME} \
-o /home/user/python-data/label_studio_import/${DATASET_NAME}.json \
--image-ext .png --out-type annotations \
--image-root-url /data/local-files/?d=/home/user/python-data/datasets/all/images/

# mv images to all/images

mv -v $REPO_ROOT/python/datasets/incoming/save/* $REPO_ROOT/python/datasets/all/images/