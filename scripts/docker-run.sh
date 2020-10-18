docker stop pontjes
docker rm -f pontjes
docker run -d -p 6376:6376 --net host --name pontjes --restart always pontjes:1
