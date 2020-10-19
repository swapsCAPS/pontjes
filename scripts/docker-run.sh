TAG=$1
docker stop pontjes
docker rm -f pontjes
docker run -d -p 6376:6376 --name pontjes --restart always pontjes:$TAG
