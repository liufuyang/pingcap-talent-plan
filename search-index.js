var N=null,E="",T="t",U="u",searchIndex={};
var R=["string","result","option","kvstore","kvstorepingcap","tosocketaddrs","remove","try_from","try_into","borrow","borrow_mut","type_id","typeid","Sets the value of a string key to a string.","Gets the string value of a given string key.","Removes a given key.","kvserror","formatter","backtrace","KvsError","KvsClient","KvStorePingCap","SledKvsEngine","KvsServer","KvsEngine"];
searchIndex["building_blocks2"]={"doc":E,"i":[],"p":[]};
searchIndex["kvs"]={"doc":"A simple key/value store.","i":[[3,R[20],"kvs","Key value store client",N,N],[3,"KvStore",E,"The struct to hold key value pairs. Currently it uses…",N,N],[3,R[21],E,"The `KvStorePingCap` stores string key/value pairs.",N,N],[3,R[22],E,"Wrapper of `sled::Db`",N,N],[3,R[23],E,"The server of a key value store.",N,N],[4,R[19],E,"Error type for kvs",N,N],[13,"Io",E,"IO error",0,N],[13,"Serde",E,"Serialization or deserialization error",0,N],[13,"KeyNotFound",E,"Removing non-existent key error",0,N],[13,"UnexpectedCommandType",E,"Unexpected command type error. It indicated a corrupted…",0,N],[13,"Utf8",E,"Key or value is invalid UTF-8 sequence",0,N],[13,"Sled",E,"Sled error",0,N],[13,"StringError",E,"Error with a string message",0,N],[13,"ParseIntError",E,"parse int error",0,N],[11,"connect",E,"Connect to `addr` to access `KvsServer`.",1,[[[R[5]]],[R[1]]]],[11,"get",E,"Get the value of a given key from the server.",1,[[["self"],[R[0]]],[[R[2],[R[0]]],[R[1],[R[2]]]]]],[11,"set",E,"Set the value of a string key in the server.",1,[[["self"],[R[0]]],[R[1]]]],[11,R[6],E,"Remove a string key in the server.",1,[[["self"],[R[0]]],[R[1]]]],[11,"open",E,"Create or scan a logfile and create a KvStore from it.",2,[[],[[R[1],[R[3]]],[R[3]]]]],[11,"open",E,"Opens a `KvStorePingCap` with the given path.",3,[[],[[R[4]],[R[1],[R[4]]]]]],[11,"compact",E,"Clears stale entries in the log.",3,[[["self"]],[R[1]]]],[11,"new",E,"Creates a `SledKvsEngine` from `sled::Db`.",4,[[["db"]],["self"]]],[11,"new",E,"Create a `KvsServer` with a given storage engine.",5,[[["e"]],["self"]]],[11,"run",E,"Run the server listening on the given address",5,[[[R[5]]],[R[1]]]],[6,"Result",E,"Result type for kvs",N,N],[8,R[24],E,"Trait for a key value storage engine.",N,N],[10,"set",E,R[13],6,[[["self"],[R[0]]],[R[1]]]],[10,"get",E,R[14],6,[[["self"],[R[0]]],[[R[2],[R[0]]],[R[1],[R[2]]]]]],[10,R[6],E,R[15],6,[[["self"],[R[0]]],[R[1]]]],[11,"from",E,E,1,[[[T]],[T]]],[11,"into",E,E,1,[[],[U]]],[11,R[7],E,E,1,[[[U]],[R[1]]]],[11,R[8],E,E,1,[[],[R[1]]]],[11,R[9],E,E,1,[[["self"]],[T]]],[11,R[10],E,E,1,[[["self"]],[T]]],[11,R[11],E,E,1,[[["self"]],[R[12]]]],[11,"from",E,E,2,[[[T]],[T]]],[11,"into",E,E,2,[[],[U]]],[11,R[7],E,E,2,[[[U]],[R[1]]]],[11,R[8],E,E,2,[[],[R[1]]]],[11,R[9],E,E,2,[[["self"]],[T]]],[11,R[10],E,E,2,[[["self"]],[T]]],[11,R[11],E,E,2,[[["self"]],[R[12]]]],[11,"from",E,E,3,[[[T]],[T]]],[11,"into",E,E,3,[[],[U]]],[11,R[7],E,E,3,[[[U]],[R[1]]]],[11,R[8],E,E,3,[[],[R[1]]]],[11,R[9],E,E,3,[[["self"]],[T]]],[11,R[10],E,E,3,[[["self"]],[T]]],[11,R[11],E,E,3,[[["self"]],[R[12]]]],[11,"from",E,E,4,[[[T]],[T]]],[11,"into",E,E,4,[[],[U]]],[11,"to_owned",E,E,4,[[["self"]],[T]]],[11,"clone_into",E,E,4,[[["self"],[T]]]],[11,R[7],E,E,4,[[[U]],[R[1]]]],[11,R[8],E,E,4,[[],[R[1]]]],[11,R[9],E,E,4,[[["self"]],[T]]],[11,R[10],E,E,4,[[["self"]],[T]]],[11,R[11],E,E,4,[[["self"]],[R[12]]]],[11,"from",E,E,5,[[[T]],[T]]],[11,"into",E,E,5,[[],[U]]],[11,R[7],E,E,5,[[[U]],[R[1]]]],[11,R[8],E,E,5,[[],[R[1]]]],[11,R[9],E,E,5,[[["self"]],[T]]],[11,R[10],E,E,5,[[["self"]],[T]]],[11,R[11],E,E,5,[[["self"]],[R[12]]]],[11,"from",E,E,0,[[[T]],[T]]],[11,"into",E,E,0,[[],[U]]],[11,"to_string",E,E,0,[[["self"]],[R[0]]]],[11,R[7],E,E,0,[[[U]],[R[1]]]],[11,R[8],E,E,0,[[],[R[1]]]],[11,R[9],E,E,0,[[["self"]],[T]]],[11,R[10],E,E,0,[[["self"]],[T]]],[11,R[11],E,E,0,[[["self"]],[R[12]]]],[11,"as_fail",E,E,0,[[["self"]],["fail"]]],[11,"get",E,"Get value by a key from store",2,[[["self"],[R[0]]],[[R[2],[R[0]]],[R[1],[R[2]]]]]],[11,"set",E,"Set key value to store",2,[[["self"],[R[0]]],[R[1]]]],[11,R[6],E,"Remove key value from store",2,[[["self"],[R[0]]],[R[1]]]],[11,"set",E,R[13],3,[[["self"],[R[0]]],[R[1]]]],[11,"get",E,R[14],3,[[["self"],[R[0]]],[[R[2],[R[0]]],[R[1],[R[2]]]]]],[11,R[6],E,R[15],3,[[["self"],[R[0]]],[R[1]]]],[11,"set",E,E,4,[[["self"],[R[0]]],[R[1]]]],[11,"get",E,E,4,[[["self"],[R[0]]],[[R[2],[R[0]]],[R[1],[R[2]]]]]],[11,R[6],E,E,4,[[["self"],[R[0]]],[R[1]]]],[11,"from",E,E,0,[[["error"]],[R[16]]]],[11,"from",E,E,0,[[["error"]],[R[16]]]],[11,"from",E,E,0,[[["fromutf8error"]],[R[16]]]],[11,"from",E,E,0,[[["error"]],[R[16]]]],[11,"from",E,E,0,[[["parseinterror"]],[R[16]]]],[11,"clone",E,E,4,[[["self"]],["sledkvsengine"]]],[11,"fmt",E,E,0,[[["self"],[R[17]]],[R[1]]]],[11,"fmt",E,E,0,[[["self"],[R[17]]],[R[1]]]],[11,"name",E,E,0,[[["self"]],[["str"],[R[2],["str"]]]]],[11,"cause",E,E,0,[[["self"]],[["fail"],[R[2],["fail"]]]]],[11,R[18],E,E,0,[[["self"]],[[R[18]],[R[2],[R[18]]]]]]],"p":[[4,R[19]],[3,R[20]],[3,"KvStore"],[3,R[21]],[3,R[22]],[3,R[23]],[8,R[24]]]};
searchIndex["kvs_client"]={"doc":E,"i":[],"p":[]};
searchIndex["kvs_server"]={"doc":E,"i":[],"p":[]};
initSearch(searchIndex);addSearchOptions(searchIndex);