use core::sync::atomic::AtomicU64;
use lazy_static::lazy_static;
use prometheus_client::encoding::text::encode;
use prometheus_client::metrics::gauge::Gauge;
use prometheus_client::registry::Registry;
use redis::InfoDict;
use std::collections::HashMap;
use std::collections::hash_map::Entry;

// "acl_access_denied_auth": simple-string("0")
// "acl_access_denied_channel": simple-string("0")
// "acl_access_denied_cmd": simple-string("0")
// "acl_access_denied_key": simple-string("0")
// "active_defrag_hits": simple-string("0")
// "active_defrag_key_hits": simple-string("0")
// "active_defrag_key_misses": simple-string("0")
// "active_defrag_misses": simple-string("0")
// "active_defrag_running": simple-string("0")
// "allocator_active": simple-string("8912896")
// "allocator_allocated": simple-string("1281296")
// "allocator_frag_bytes": simple-string("7631600")
// "allocator_frag_ratio": simple-string("6.96")
// "allocator_resident": simple-string("10682368")
// "allocator_rss_bytes": simple-string("1769472")
// "allocator_rss_ratio": simple-string("1.20")
// "aof_current_rewrite_time_sec": simple-string("-1")
// "aof_enabled": simple-string("0")
// "aof_last_bgrewrite_status": simple-string("ok")
// "aof_last_cow_size": simple-string("0")
// "aof_last_rewrite_time_sec": simple-string("-1")
// "aof_last_write_status": simple-string("ok")
// "aof_rewrite_in_progress": simple-string("0")
// "aof_rewrite_scheduled": simple-string("0")
// "aof_rewrites": simple-string("0")
// "aof_rewrites_consecutive_failures": simple-string("0")
// "arch_bits": simple-string("64")
// "async_loading": simple-string("0")
// "atomicvar_api": simple-string("c11-builtin")
// "blocked_clients": simple-string("0")
// "client_recent_max_input_buffer": simple-string("8")
// "client_recent_max_output_buffer": simple-string("0")
// "clients_in_timeout_table": simple-string("0")
// "cluster_connections": simple-string("0")
// "cluster_enabled": simple-string("0")
// "config_file": simple-string("")
// "configured_hz": simple-string("10")
// "connected_clients": simple-string("46")
// "connected_slaves": simple-string("0")
// "current_active_defrag_time": simple-string("0")
// "current_cow_peak": simple-string("0")
// "current_cow_size": simple-string("0")
// "current_cow_size_age": simple-string("0")
// "current_eviction_exceeded_time": simple-string("0")
// "current_fork_perc": simple-string("0.00")
// "current_save_keys_processed": simple-string("0")
// "current_save_keys_total": simple-string("0")
// "dump_payload_sanitizations": simple-string("0")
// "eventloop_cycles": simple-string("939198")
// "eventloop_duration_cmd_sum": simple-string("5048")
// "eventloop_duration_cmd_sum": simple-string("5436")
// "eventloop_duration_sum": simple-string("248709105")
// "evicted_clients": simple-string("0")
// "evicted_keys": simple-string("0")
// "executable": simple-string("/data/valkey-server")
// "expire_cycle_cpu_milliseconds": simple-string("2581")
// "expired_keys": simple-string("0")
// "expired_stale_perc": simple-string("0.00")
// "expired_time_cap_reached_count": simple-string("0")
// "gcc_version": simple-string("12.2.0")
// "hz": simple-string("10")
// "instantaneous_eventloop_cycles_per_sec": simple-string("9")
// "instantaneous_eventloop_duration_usec": simple-string("192")
// "instantaneous_input_kbps": simple-string("0.00")
// "instantaneous_input_repl_kbps": simple-string("0.00")
// "instantaneous_ops_per_sec": simple-string("0")
// "instantaneous_output_kbps": simple-string("0.00")
// "instantaneous_output_repl_kbps": simple-string("0.00")
// "io_threaded_reads_processed": simple-string("0")
// "io_threaded_writes_processed": simple-string("0")
// "io_threads_active": simple-string("0")
// "keyspace_hits": simple-string("0")
// "keyspace_misses": simple-string("0")
// "latest_fork_usec": simple-string("0")
// "lazyfree_pending_objects": simple-string("0")
// "lazyfreed_objects": simple-string("0")
// "listener0": simple-string("name=tcp,bind=*,bind=-::*,port=6379")
// "loading": simple-string("0")
// "lru_clock": simple-string("12639546")
// "master_failover_state": simple-string("no-failover")
// "master_repl_offset": simple-string("0")
// "master_replid": simple-string("1eeca0a6742375d28b14013a87ee7cd872a9a417")
// "master_replid2": simple-string("0000000000000000000000000000000000000000")
// "maxclients": simple-string("10000")
// "maxmemory": simple-string("0")
// "maxmemory_human": simple-string("0B")
// "maxmemory_policy": simple-string("noeviction")
// "mem_allocator": simple-string("jemalloc-5.3.0")
// "mem_aof_buffer": simple-string("0")
// "mem_clients_normal": simple-string("84832")
// "mem_clients_slaves": simple-string("0")
// "mem_cluster_links": simple-string("0")
// "mem_fragmentation_bytes": simple-string("8301816")
// "mem_fragmentation_ratio": simple-string("8.76")
// "mem_not_counted_for_evict": simple-string("0")
// "mem_replication_backlog": simple-string("0")
// "mem_total_replication_buffers": simple-string("0")
// "migrate_cached_sockets": simple-string("0")
// "module_fork_in_progress": simple-string("0")
// "module_fork_last_cow_size": simple-string("0")
// "monotonic_clock": simple-string("POSIX clock_gettime")
// "multiplexing_api": simple-string("epoll")
// "number_of_cached_scripts": simple-string("0")
// "number_of_functions": simple-string("0")
// "number_of_libraries": simple-string("0")
// "os": simple-string("Linux 6.9.12-200.fc40.aarch64 aarch64")
// "process_id": simple-string("1")
// "process_supervised": simple-string("no")
// "pubsub_channels": simple-string("0")
// "pubsub_patterns": simple-string("0")
// "pubsubshard_channels": simple-string("0")
// "rdb_bgsave_in_progress": simple-string("0")
// "rdb_changes_since_last_save": simple-string("0")
// "rdb_current_bgsave_time_sec": simple-string("-1")
// "rdb_last_bgsave_status": simple-string("ok")
// "rdb_last_bgsave_time_sec": simple-string("-1")
// "rdb_last_cow_size": simple-string("0")
// "rdb_last_load_keys_expired": simple-string("0")
// "rdb_last_load_keys_loaded": simple-string("0")
// "rdb_last_save_time": simple-string("1740595239")
// "rdb_saves": simple-string("0")
// "redis_build_id": simple-string("85194c11cc4d34e4")
// "redis_git_dirty": simple-string("0")
// "redis_git_sha1": simple-string("00000000")
// "redis_mode": simple-string("standalone")
// "redis_version": simple-string("7.2.4")
// "rejected_connections": simple-string("0")
// "repl_backlog_active": simple-string("0")
// "repl_backlog_first_byte_offset": simple-string("0")
// "repl_backlog_histlen": simple-string("0")
// "repl_backlog_size": simple-string("1048576")
// "reply_buffer_expands": simple-string("0")
// "reply_buffer_shrinks": simple-string("44")
// "role": simple-string("master")
// "rss_overhead_bytes": simple-string("-1310720")
// "rss_overhead_ratio": simple-string("0.88")
// "run_id": simple-string("ee997ade2683f5d868ef19764815026bab40821f")
// "second_repl_offset": simple-string("-1")
// "server_name": simple-string("valkey")
// "server_time_usec": simple-string("1740692794192456")
// "server_time_usec": simple-string("1740692794192851")
// "slave_expires_tracked_keys": simple-string("0")
// "sync_full": simple-string("0")
// "sync_partial_err": simple-string("0")
// "sync_partial_ok": simple-string("0")
// "tcp_port": simple-string("6379")
// "total_active_defrag_time": simple-string("0")
// "total_blocking_keys": simple-string("0")
// "total_blocking_keys_on_nokey": simple-string("0")
// "total_commands_processed": simple-string("136")
// "total_commands_processed": simple-string("137")
// "total_connections_received": simple-string("46")
// "total_error_replies": simple-string("0")
// "total_eviction_exceeded_time": simple-string("0")
// "total_forks": simple-string("0")
// "total_net_input_bytes": simple-string("5736")
// "total_net_input_bytes": simple-string("5750")
// "total_net_output_bytes": simple-string("235333")
// "total_net_repl_input_bytes": simple-string("0")
// "total_net_repl_output_bytes": simple-string("0")
// "total_reads_processed": simple-string("91")
// "total_reads_processed": simple-string("92")
// "total_system_memory": simple-string("2043928576")
// "total_system_memory_human": simple-string("1.90G")
// "total_writes_processed": simple-string("90")
// "tracking_clients": simple-string("0")
// "tracking_total_items": simple-string("0")
// "tracking_total_keys": simple-string("0")
// "tracking_total_prefixes": simple-string("0")
// "unexpected_error_replies": simple-string("0")
// "uptime_in_days": simple-string("1")
// "uptime_in_seconds": simple-string("97555")
//
// "used_cpu_sys": simple-string("151.185961")
// "used_cpu_sys_children": simple-string("0.000491")
// "used_cpu_sys_main_thread": simple-string("151.185210")
// "used_cpu_sys_main_thread": simple-string("151.185238")
// "used_cpu_user": simple-string("97.857111")
// "used_cpu_user": simple-string("97.857156")
// "used_cpu_user_children": simple-string("0.001158")
// "used_cpu_user_main_thread": simple-string("97.856607")
// "used_cpu_user_main_thread": simple-string("97.856625")
//
//
// "valkey_version": simple-string("7.2.8")

enum MetricKind {
    GaugeI64,
    GaugeF64,
    Counter,
}

struct MetricDesc {
    kind: MetricKind,
    name: &'static str,
    descr: &'static str,
}

macro_rules! metric_desc {
    ($kind:ident, $name:ident, $descr:literal) => {
        MetricDesc {
            kind: MetricKind::$kind,
            name: stringify!($name),
            descr: $descr,
        }
    };
}

lazy_static! {
    static ref METRICS_DESC: Vec<MetricDesc> = {
        vec![
            // GaugeI64
            metric_desc!(GaugeI64, used_memory, "used_memory"),
            metric_desc!(GaugeI64, used_memory_dataset, "used_memory_dataset"),
            metric_desc!(GaugeI64, used_memory_functions, "used_memory_functions"),
            metric_desc!(GaugeI64, used_memory_lua, "used_memory_lua"),
            metric_desc!(GaugeI64, used_memory_overhead, "used_memory_overhead"),
            metric_desc!(GaugeI64, used_memory_peak, "used_memory_peak"),
            metric_desc!(GaugeI64, used_memory_rss, "used_memory_rss"),
            metric_desc!(GaugeI64, used_memory_scripts, "used_memory_scripts"),
            metric_desc!(
                GaugeI64,
                used_memory_scripts_eval,
                "used_memory_scripts_eval"
            ),
            metric_desc!(GaugeI64, used_memory_startup, "used_memory_startup"),
            metric_desc!(GaugeI64, used_memory_vm_eval, "used_memory_vm_eval"),
            metric_desc!(
                GaugeI64,
                used_memory_vm_functions,
                "used_memory_vm_functions"
            ),
            metric_desc!(GaugeI64, used_memory_vm_total, "used_memory_vm_total"),

            // GaugeF64
            metric_desc!(GaugeF64, used_cpu_sys, "used_cpu_sys"),
            metric_desc!(GaugeF64, used_cpu_sys_children, "used_cpu_sys_children"),
            metric_desc!(GaugeF64, used_cpu_sys_main_thread, "used_cpu_sys_main_thread"),
            metric_desc!(GaugeF64, used_cpu_sys_main_thread, "used_cpu_sys_main_thread"),
            metric_desc!(GaugeF64, used_cpu_user, "used_cpu_user"),
            metric_desc!(GaugeF64, used_cpu_user, "used_cpu_user"),
            metric_desc!(GaugeF64, used_cpu_user_children, "used_cpu_user_children"),
            metric_desc!(GaugeF64, used_cpu_user_main_thread, "used_cpu_user_main_thread"),
            metric_desc!(GaugeF64, used_cpu_user_main_thread, "used_cpu_user_main_thread"),
        ]
    };
}

#[derive(Debug, Default)]
pub struct Metrics {
    registry: Registry,
    gauge_i64: HashMap<String, Gauge>,
    gauge_f64: HashMap<String, Gauge<f64, AtomicU64>>,
}

impl Metrics {
    pub fn update(&mut self, info: &InfoDict) {
        for x in METRICS_DESC.iter() {
            match x.kind {
                MetricKind::GaugeI64 => {
                    if let Some(v) = info.get(x.name) {
                        match self.gauge_i64.entry(x.name.to_string()) {
                            Entry::Vacant(e) => {
                                let gauge = Gauge::default();
                                gauge.set(v);
                                self.registry.register(x.name, x.descr, gauge.clone());
                                e.insert(gauge);
                            }
                            Entry::Occupied(mut e) => {
                                e.get_mut().set(v);
                            }
                        };
                    };
                }
                MetricKind::GaugeF64 => {
                    if let Some(v) = info.get(x.name) {
                        match self.gauge_f64.entry(x.name.to_string()) {
                            Entry::Vacant(e) => {
                                let gauge = Gauge::default();
                                gauge.set(v);
                                self.registry.register(x.name, x.descr, gauge.clone());
                                e.insert(gauge);
                            }
                            Entry::Occupied(mut e) => {
                                e.get_mut().set(v);
                            }
                        };
                    };
                }
                MetricKind::Counter => {
                    unimplemented!()
                }
            }
        }
    }

    pub fn encode(&self, buf: &mut String) {
        encode(buf, &self.registry).unwrap();
    }
}
