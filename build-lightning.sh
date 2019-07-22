#!/usr/bin/env bash

RELEASE=$(cat release)

wget "https://ftp.gnu.org/gnu/lightning/$RELEASE.tar.gz" -O $OUT_DIR/$RELEASE.tar.gz
tar xvf $OUT_DIR/$RELEASE.tar.gz -C $OUT_DIR/

PREFIX=$1
(
    cd $OUT_DIR/$RELEASE
    ./configure --prefix=$PREFIX --enable-disassembler
    make -j4
    make install
)


