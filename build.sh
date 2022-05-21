#!/bin/bash

echo "pull the latest image from registry"

docker pull cristian44/public-chat:latest

echo "Stop the old server"

docker stop public-chat

echo "Remove the old server"

docker rm public-chat

echo "Run the new server"

docker run -d -p 8080:8080 --name public-chat cristian44/public-chat:latest