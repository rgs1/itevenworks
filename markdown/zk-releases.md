## Validating ZooKeeper Releases ##

A quick way to setup containers in Fedora is via <a href="http://www.freedesktop.org/software/systemd/man/systemd-nspawn.html" target="_blank">systemd-nspawn</a> (a nice intro post about it <a href="http://0pointer.net/blog/systemd-for-administrators-part-xxi.html" target="_blank">here</a>):

Enable networkd on the host so that the NSS magic described below works (for resolving containers):

```shell
$ sudo systemctl enable systemd-networkd
$ sudo systemctl start systemd-networkd
$ sudo systemctl enable systemd-resolved
$ sudo systemctl start systemd-resolved
```

More info on systemd-network <a href="https://wiki.archlinux.org/index.php/Systemd-networkd" target="_blank">here</a>.

To be able to resolve containers from the host modify /etc/nsswitch.conf so that you have:
```conf
#hosts:     db files nisplus nis dns
hosts:      files mymachines mdns4_minimal [NOTFOUND=return] dns myhostname
```

You'll also want to enable forwarding + masquerading for containers:
```shell
$ sudo sysctl net.ipv4.ip_forward=1
$ sudo iptables -t nat -A POSTROUTING -o wlp1s0 -j MASQUERADE
```

First, prepare the template container:

```shell
$ sudo yum -y --releasever=21 --nogpg --installroot=/srv/virt-00 \
  --disablerepo='*' --enablerepo=fedora groupinstall 'Web Server'
```

Set the root password:
```shell
$ sudo systemd-nspawn --machine=virt-00 --network-veth -D /srv/virt-00
```

Now, start the template container and get it setup for ZK:

```shell
$ sudo systemd-nspawn --machine=virt-00 --network-veth -bD /srv/virt-00
# ...
$ yum install java ant
# download the ZK release you'd like to test
$ url=http://people.apache.org/~michim/zookeeper-3.5.1-alpha-candidate-0/zookeeper-3.5.1-alpha.tar.gz
$ wget $url
# build it
$ export JAVA_TOOL_OPTIONS=-Dfile.encoding=UTF8
$ tar xzvf zookeeper-3.5.1-alpha.tar.gz
$ cd zookeeper-3.5.1-alpha
$ ant jar
```

Great, the template container is ready. Now lets copy it and spawn the other containers (for participants/observers). Note that machinectl, used to manage containers, will soon have support for cloning machines (here is the <a href="http://cgit.freedesktop.org/systemd/systemd/commit/?id=ebd93cb684806ac0f352139e69ac8f53eb49f5e4">commit</a>). For now, we'll have to do it manually:

```shell
$ cd /srv
$ for i in $(seq 1 5) ; do sudo cp -r virt-00 virt-0${i} ; done
# start all
$ function start() { sudo systemd-nspawn --machine=virt-${1} --network-veth -bD /srv/virt-${1}; }
# ... each one on its own terminal ...
$ start 01
```

Now, we need to find the IP addresses for all containers and generate a config (zoo.cfg). For this, we can use the following Python script:

```python
# gen-zoo-cfg.py
#!/usr/bin/python

from __future__ import print_function

import socket
import sys


def ips(prefix, hosts, port=2181):
    for host in hosts:
        addrs = socket.getaddrinfo(host, port)
        for addr in addrs:
            ip = addr[4][0]
            if ip.startswith(prefix):
                yield ip
                break


def conf(members):
    cstr = """

tickTime=2000
initLimit=10
syncLimit=5
dataDir=/tmp/zookeeper
clientPort=2181
"""
    members_conf = []
    for mid, ip in members:
        mline = "server.%d=%s:2889:3888:participant;0.0.0.0:2181" % (mid, ip)
        members_conf.append(mline)

    return "\n".join(members_conf) + cstr


if __name__ == '__main__':
    idip = enumerate(ips('10.', sys.argv[1:]))
    members = [(mid, ip) for mid, ip in idip]
    cstr = conf(members)
    print(cstr)
```

Finally, we need to distribute the config file (zoo.cfg) to every container and assign their ids (by creating a file called myid with it inside the data dir). The way I do it is by serving the config file via HTTP from the host hosting the containers:

```shell

$ cat <<EOF > /tmp/zoo.cfg
server.0=10.0.0.2:2889:3888:participant;0.0.0.0:2181
server.1=10.0.0.18:2889:3888:participant;0.0.0.0:2181
server.2=10.0.0.34:2889:3888:participant;0.0.0.0:2181
server.3=10.0.0.50:2889:3888:participant;0.0.0.0:2181
server.4=10.0.0.66:2889:3888:participant;0.0.0.0:2181
server.5=10.0.0.82:2889:3888:observer;0.0.0.0:2181

tickTime=2000
initLimit=10
syncLimit=5
dataDir=/tmp/zookeeper
clientPort=2181
EOF

$ cd /tmp
$ python -m SimpleHTTPServer 8080
```

Then you can fetch it from all containers at once. If you use tmux — and you should — you can use this <a href="http://stackoverflow.com/questions/16325449/how-to-send-a-command-to-all-panes-in-tmux" target="_blank">little trick</a> to run multiple commands on all containers.

Or, given the NSS magic described above which allows you to resolve containers, you can loop and ssh into all of them to fetch the configs:

```shell
$ machinectl
MACHINE                          CONTAINER SERVICE
virt-05                          container nspawn
virt-04                          container nspawn
virt-03                          container nspawn
virt-02                          container nspawn
virt-01                          container nspawn
virt-00                          container nspawn

6 machines listed.

$ for i in $(seq 0 5) ; do
   ssh root@virt-0${i} 'wget http://10.2.2.2:8080/zoo.cfg -O /root/zookeeper-3.5.1-alpha/conf'
  done
```

Great! Only one more thing left, which is to assign ids to each box. Yet another loop:

```shell
# create the data dir (defaults to /tmp/zookeeper)
$ for i in $(seq 0 5) ; do
   ssh root@virt-0${i} 'mkdir /tmp/zookeeper'
  done

# assign ids
$ for i in $(seq 0 5) ; do
   ssh root@virt-0${i} 'echo $(hostname | cut -d- -f2 | sed "s#^0##g") > /tmp/zookeeper/myid'
  done
```

And now, we start the cluster:

```shell
$ for i in $(seq 0 5) ; do
   ssh root@virt-0${i} 'bash -x /root/zookeeper-3.5.1-alpha/bin/zkServer.sh start'
  done
```

To test the cluster from the host, we can use <a href="https://github.com/rgs1/zk_shell">zk-shell</a>:

```shell
$ zk-shell virt-00
Welcome to zk-shell (1.0.05)
(CONNECTING) />
(CONNECTED) /> loop 100 0 'create test_ "" true true '
(CONNECTED) /> exists /
Stat(
  czxid=0
  mzxid=0
  ctime=0
  mtime=0
  version=0
  cversion=99
  aversion=0
  ephemeralOwner=0x0
  dataLength=0
  numChildren=101
  pzxid=4294967397
)
(CONNECTED) />
```
