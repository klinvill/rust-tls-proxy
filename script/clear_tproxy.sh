#!/bin/bash

mark=8 # packets marked by iptables are routed according to ip rule and ip route
entryn=9

# flush existing modifications
iptables -t mangle -F
ip rule delete fwmark $mark
ip route del local default dev lo table $entryn 
