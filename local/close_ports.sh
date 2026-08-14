# Close syre ports.
SYRE_PROCESS_NAME=syre
SYRE_SERVER_PORT=7048

pid=$(lsof -i :$SYRE_SERVER_PORT | grep $SYRE_PROCESS_NAME | cut -f 2 -w)
if [ -n "$pid" ]; then
	kill $pid
fi
