CONFIG=00/default.toml nohup ./rust_storage >> log/0.log &
CONFIG=01/default.toml nohup ./rust_storage >> log/1.log &
CONFIG=02/default.toml nohup ./rust_storage >> log/2.log &
CONFIG=03/default.toml nohup ./rust_storage >> log/3.log &
CONFIG=04/default.toml nohup ./rust_storage >> log/4.log &
CONFIG=05/default.toml nohup ./rust_storage >> log/5.log &
tail -f log/*.log