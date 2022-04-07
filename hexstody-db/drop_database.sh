USERNAME=${1:-hexstody}
DATABASE=${2:-hexstody}
HOST=${3:-127.0.0.1}
export PGPASSWORD=${4:-hexstody}
export DATABASE_URL=postgres://$USERNAME@$HOST/$DATABASE

sqlx database drop