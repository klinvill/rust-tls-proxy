#!/bin/bash

mark=8 # packets marked by iptables are routed according to ip rule and ip route
entryn=9
http_port=9980 # destination server port
proxy_redir_port=8080 # proxy server listener application port

# add iptable rules to mark all packets for port 1234

# the following command forces us to listen on the same port as the destination
# iptables -t mangle -A PREROUTING -p tcp --dport $srcport -j MARK --set-mark $mark

# the following command erases the original port information
# iptables -t nat -A PREROUTING -p tcp -j REDIRECT --to-port $lstnport

# but we can do both with a TPROXY rule
iptables -t mangle -A PREROUTING -p tcp --dport $http_port -j TPROXY \
	--tproxy-mark $mark/$mark --on-port $proxy_redir_port

# route packets marked with 8 to localhost according to table 9
ip rule add fwmark $mark table $entryn
ip route add local default dev lo table $entryn

# verify settings correct
iptables -t mangle -L -v
ip rule
ip route list table $entryn
