# get version from cargo
VERSION=$(grep '^version' ../Cargo.toml | grep -o '[0-9]\.[0-9]\.[0-9]')
echo "Building container with tag lokmeinmatz/what-cloud-backend:$VERSION"
docker build -t lokmeinmatz/what-cloud-backend:$VERSION .
# check build status
if [[$? -ne 0]]; then
    echo "Build failed :("
fi