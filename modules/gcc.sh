#!/bin/bash

set -xe

export GCC_VERSIONS="7.4.0 8.3.0 9.2.0"

for VERSION in $GCC_VERSIONS; do
    curl -fSsL "https://ftpmirror.gnu.org/gcc/gcc-$VERSION/gcc-$VERSION.tar.gz" -o /tmp/gcc-$VERSION.tar.gz
    mkdir /tmp/gcc-$VERSION
    tar -xf /tmp/gcc-$VERSION.tar.gz -C /tmp/gcc-$VERSION --strip-components=1
    rm /tmp/gcc-$VERSION.tar.gz
    cd /tmp/gcc-$VERSION
    ./contrib/download_prerequisites
    { rm *.tar.* || true; }
    tmpdir="$(mktemp -d)"
    cd "$tmpdir";
    
    if [ $VERSION = "9.2.0" ]; then
        ENABLE_FORTRAN=",fortran";
    else
        ENABLE_FORTRAN="";
    fi;
    
    /tmp/gcc-$VERSION/configure --disable-multilib --enable-languages=c,c++$ENABLE_FORTRAN --prefix=/usr/local/gcc-$VERSION
    make -j$(nproc)
    make -j$(nproc) install-strip
    rm -rf /tmp/*;
done
