echo ${REPO_ROOT}
# this will break label studio, whose data is in /data so we only do /python
sudo chown eric:eric -R ${REPO_ROOT}/python/datasets