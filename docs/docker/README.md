# 创建镜像

预先装好docker，在项目根路径执行以下命令创建镜像
```shell
docker build -f Dockerfile -t rust_storage .
```

# 创建bridge网络

```shell
docker network create storage-net
```

# 启动服务

进入docs/docker目录，执行以下命令创建服务
```shell
docker run -it --name storage-0 --network storage-net --network-alias storage-0 -p 3000:3000 -p 3100:3100 -v $PWD/config:/config:rw -e CONFIG=/config/config_0.toml -d rust_storage
docker run -it --name storage-1 --network storage-net --network-alias storage-1 -p 3001:3001 -p 3101:3101 -v $PWD/config:/config:rw -e CONFIG=/config/config_1.toml -d rust_storage
docker run -it --name storage-2 --network storage-net --network-alias storage-2 -p 3002:3002 -p 3102:3102 -v $PWD/config:/config:rw -e CONFIG=/config/config_2.toml -d rust_storage
docker run -it --name storage-3 --network storage-net --network-alias storage-3 -p 3003:3003 -p 3103:3103 -v $PWD/config:/config:rw -e CONFIG=/config/config_3.toml -d rust_storage
docker run -it --name storage-4 --network storage-net --network-alias storage-4 -p 3004:3004 -p 3104:3104 -v $PWD/config:/config:rw -e CONFIG=/config/config_4.toml -d rust_storage
docker run -it --name storage-5 --network storage-net --network-alias storage-5 -p 3005:3005 -p 3105:3105 -v $PWD/config:/config:rw -e CONFIG=/config/config_5.toml -d rust_storage
```
