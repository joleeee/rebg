#/bin/sh

for platform in arm64 amd64
do
	echo "building ${platform}"
	docker build --platform linux/${platform} . -t rebg:${platform}
done
