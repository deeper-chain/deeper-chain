ps -ef|grep deeper-chain |grep -v grep|cut -c 9-15|xargs kill -9
rm -rf /tmp/*
