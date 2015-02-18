### Software Requirements and Specifications: A Lexicon of Practice, Principles and Prejudices

I first heard about <a href="http://en.wikipedia.org/wiki/Michael_A._Jackson" target="_blank">Michael A. Jackson</a> whilst listening to <a href="http://en.wikipedia.org/wiki/Rob_Pike" target="_blank">Rob Pike</a>. I got interested in some of his ideas so I started reading <a href="http://www.amazon.com/Software-Requirements-Specifications-Principles-Prejudices/dp/0201877120" target="_blank">Software Requirements and Specifications: A Lexicon of Practice, Principles and Prejudices</a> and so far I could only wished I had read this book before: lots of very good points!

### A ZooKeeper Observer that can follow multiple leaders

Imagine that you could have mount points in [ZooKeeper](http://zookeeper.apache.org/). In your config you would associate a cluster config with a mount-point perhaps doing something like:


```conf
cluster1 = 10.0.1.1:2889:3889;10.0.1.2:2889:3889;10.0.1.3:2889:3889
cluster2 = 10.0.2.4:2889:3889;10.0.2.5:2889:3889;10.0.2.6:2889:3889

cluster1:/configs /cluster1/configs  # follows cluster 1
cluster2:/configs /cluster2/configs  # ditto for cluster 2
```


This would allow to effectively parallelize [ZAB](http://www.tcs.hut.fi/Studies/T-79.5001/reports/2012-deSouzaMedeiros.pdf) operations, which by definition are only linear. Of course, it means breaking ZooKeeper semantics across different paths in /. Though, in practice, it's unlikely that your apps need full semantics across *all* of the / namespace, anyway. Then, these operations would be close to parallel (i.e.: you stil serialize within your session):


```ruby
zk = ZkClient.new(:cluster => "localhost:2181")
zk.create("/cluster1/configs/myconfig", json_blob)
zk.create("/cluster2/configs/myconfig", json_blob)
```

This could be a huge improvement for multi-tenant ZooKeeper setups, which could still share the costs of Observers.

### pcap-ng support in Linux

Turns out that Apple already supports <a href="http://www.opensource.apple.com/source/libpcap/libpcap-42/libpcap/pcapng.c?txt" target="_blank">pcap-ng</a>. The full RFC for pcap-ng is <a href="http://www.winpcap.org/ntar/draft/PCAP-DumpFileFormat.html" target="_blank">here</a>. <a href="http://www.tcpdump.org/" target="_blank">libpcap</a>, actually does have some support for it but the most interesting bits (i.e.: process event blocks) are still missing. pcap is very useful to understand what's going on in your network, so pcap-ng support would indeed be great.
