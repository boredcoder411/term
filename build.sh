#! /bin/bash

if [ -d "build" ]; then
  rm -rf build
fi

mkdir build

cd backend
go build server.go
mv server ../build/

cd ../frontend
npm install
npm run build

mv dist ../build

cd ..
