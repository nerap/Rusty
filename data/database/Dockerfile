# Use the official TimescaleDB image
FROM timescale/timescaledb:latest-pg16

# Copy migration scripts
COPY migrations/ /docker-entrypoint-initdb.d/

COPY <<EOF /etc/postgresql/postgresql.conf
# Extension loading
shared_preload_libraries = 'timescaledb'

# Performance tuning
shared_buffers = 2GB
effective_cache_size = 6GB
maintenance_work_mem = 512MB
max_worker_processes = 8
max_parallel_workers = 8
listen_addresses = '*'
EOF

# # Set the custom PostgreSQL configuration file as the default
CMD ["postgres", "-c", "config_file=/etc/postgresql/postgresql.conf"]

# Expose PostgreSQL port
EXPOSE 5432

