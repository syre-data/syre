# Close syre ports.
SYRE_SERVER_PROCESS_NAME="trunk"
SYRE_SERVER_PORT=1420

pid=$(lsof -i :$SYRE_SERVER_PORT | grep $SYRE_SERVER_PROCESS_NAME | cut -f 2 -w)
if [ -n "$pid" ]; then
	kill $pid
fi
