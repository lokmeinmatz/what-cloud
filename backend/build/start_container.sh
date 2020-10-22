#!/bin/bash

IMAGE_VERSION="0.1.2"
IMAGE_FULL_NAME="lokmeinmatz/what-cloud-backend:$IMAGE_VERSION"
PORT=5080
DB_DIR="/srv/what-cloud/backend"
DATA_DIR="/mnt/hdd/what-cloud"

#check if older version is running
OLD_INST_RUNNING=$(docker container inspect -f '{{.State.Status}}' what-cloud-backend)

if [[ "$OLD_INST_RUNNING" == "running" ]]; then
        echo "Version of this container is allready running, terminating and backing up old instance to what-cloud-backend-old"
        docker stop what-cloud-backend
        # delete old backup
        docker rm what-cloud-backend-old
        docker rename what-cloud-backend what-cloud-backend-old
fi

echo "starting what-cloud docker container $IMAGE_FULL_NAME"
docker run -d --name what-cloud-backend -p $PORT:80 -e ROCKET_PORT=80 -e DB_PATH=/mnt/db/backend.sqlite -e DATA_PATH=/mnt/data -v $DB_DIR:/mnt/db -v $DATA_DIR:/mnt/data $IMAGE_FULL_NAME