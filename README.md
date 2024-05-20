# Archive Witness Database Tools

'Archive Witness' is the codename for a project dedicated to archiving media pertaining to the
terrorist attacks of September 11, 2001.

This repository provides some crates for defining the database and populating it with content.

## Deployment

I'm adding a section here to leave myself some documentation on how I setup Postgres and built the
code on the server where I'm using these tools.

My server is running Arch and it has a Jenkins installation.

### Install and Configure Postgres

Postgres can be installed using `pacman`:
```
sudo pacman -S postgresql
```

The initial installation should create a `postgres` user, which can be used to configure the initial
cluster:
```
sudo -iu postgres
initdb --locale $LANG -E UTF8 -D '/var/lib/postgres/data'
exit
```

The `initdb` output contained this message:
```
Success. You can now start the database server using:

    pg_ctl -D /var/lib/postgres/data -l logfile start
```

However, it's preferable to use the service:
```
sudo systemctl start postgresql
sudo systemctl enable postgresql
```

### Configure the Archive Witness Database

Use the `postgres` user to create the database and two users for it:
```
sudo -iu postgres
[postgres@archive ~]$ psql
psql (16.2)
Type "help" for help.

postgres=# CREATE DATABASE "archive-witness";
CREATE DATABASE
postgres=# CREATE USER chris WITH PASSWORD '9EHoI04UNLab57FcQH3V';
CREATE ROLE
postgres=# GRANT ALL PRIVILEGES ON DATABASE "archive-witness" TO chris;
GRANT
postgres=# CREATE USER jenkins WITH PASSWORD 'psmOPFI9J4pJ8h5MaJcK';
CREATE ROLE
postgres=# GRANT ALL PRIVILEGES ON DATABASE "archive-witness" TO jenkins;
GRANT
postgres=# \c archive-witness
You are now connected to database "archive-witness" as user "postgres".
archive-witness=# GRANT USAGE ON SCHEMA public TO jenkins;
GRANT
archive-witness=# GRANT CREATE ON SCHEMA public TO jenkins;
GRANT
archive-witness=# GRANT ALL PRIVILEGES ON ALL TABLES IN SCHEMA public TO jenkins;
GRANT
archive-witness=# GRANT ALL PRIVILEGES ON ALL SEQUENCES IN SCHEMA public TO jenkins;
GRANT
archive-witness=# GRANT USAGE ON SCHEMA public TO chris;
GRANT
archive-witness=# GRANT CREATE ON SCHEMA public TO chris;
GRANT
archive-witness=# GRANT ALL PRIVILEGES ON ALL TABLES IN SCHEMA public TO chris;
GRANT
archive-witness=# GRANT ALL PRIVILEGES ON ALL SEQUENCES IN SCHEMA public TO chris;
GRANT
```

One user is for deployments and the other is for use with the application.

### Building with Jenkins

I am deploying the binary by building it with Jenkins, which is running on my server. My Jenkins
setup is very simple, with the master running jobs on the same machine.

The `jenkins` user needs to be setup for shell access. After that, login as `jenkins` and setup a
Rust environment.

Building the code requires a few packages:
```
sudo pacman -S clang imagemagick llvm
```

The job currently looks like this:
```bash
source $HOME/.cargo/env

export AW_DB_URL="postgres://jenkins:$JENKINS_DB_PASSWORD@localhost/archive-witness"
export YOUTUBE_DB_URL=sqlite:////mnt/sept11-archive/9-11-archive/video/yt-mirrors/videos.db

# These are necessary for the compile-time query checks.
echo "DATABASE_URL=postgres://jenkins:$JENKINS_DB_PASSWORD@localhost/archive-witness" > db/.env
echo "DATABASE_URL=sqlite:////mnt/sept11-archive/9-11-archive/video/yt-mirrors/videos.db" > db-youtube/.env

cargo install sqlx-cli
(
  cd db
  sqlx migrate run --database-url $AW_DB_URL
)

latest_tools_tag=$(git tag | grep '^tools-' | sort -V | tail -n 1)
echo "Checking out $latest_tools_tag"
git checkout $latest_tools_tag

cargo build --release
cp target/release/tools /usr/local/bin/awdb
```

Where `JENKINS_DB_PASSWORD` should be declared as a secret.

### Initial Usage

On the shell login where `awdb` will be running, declare the following environment variables:
```
AW_DB_URL=postgres://chris:<password>@localhost/archive-witness
EDITOR=vim
YOUTUBE_DB_URL=sqlite:////mnt/sept11-archive/9-11-archive/video/yt-mirrors/videos.db
```

Download the 911datasets.org torrent files:
```
mkdir -p downloads/torrents
awdb releases download-torrents --path /home/chris/downloads/torrents
```

Initialise the releases in the database:
```
awdb releases init --torrent-path /home/chris/downloads/torrents
```

The database should now be ready for use.
