#!/usr/bin/env bash

set -e

if [ $# -eq 0 ]
then
    echo "Usage: $0 <version>"
fi

VERSION="$1"

# clean any releases built with this version
rm -Rf releases/mathlingua-"$VERSION"
rm -f releases/mathlingua-"$VERSION".zip

# make sure to start with a clean build
./gradlew clean

# ensure the tests pass
./gradlew check

# build the jar
./gradlew jar

VERSION_DIR_NAME=mathlingua-"$VERSION"
VERSION_DIR=releases/"$VERSION_DIR_NAME"

# make the directory to store the release
mkdir -p "$VERSION_DIR"

MLG_PATH="$VERSION_DIR"/mlg

# copy the artifacts to the release directory
echo "#!/usr/bin/env bash" > "$MLG_PATH"
# shellcheck disable=SC2016
echo 'MLG_DIR="$( cd "$(dirname "$0")" >/dev/null 2>&1 ; pwd -P )"' >> "$MLG_PATH"
echo "java -jar \"\$MLG_DIR\"/mathlingua-$VERSION.jar \$*" >> "$MLG_PATH"
chmod +x "$MLG_PATH"
cp build/libs/mathlingua-"$VERSION".jar "$VERSION_DIR"

cd releases

# create the release
zip -r mathlingua-"$VERSION".zip "$VERSION_DIR_NAME"
