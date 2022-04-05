# Designed for debian
echo "Deploying local PostgreSQL"
pg_ctlcluster 13 main start
sudo -u postgres psql -d postgres -c "create role \"hexstody\" with login password 'hexstody';"
sudo -u postgres psql -d postgres -c "create database \"hexstody\" owner \"hexstody\";"
for f in ./hexstody-hedge-db/migrations/*.sql
do
    echo "Applying $f"
    sudo -u postgres psql -d hexstody -f $f
done
sudo -u postgres psql -d hexstody -c "GRANT ALL PRIVILEGES ON TABLE updates TO hexstody;"
export DATABASE_URL=postgres://hexstody:hexstody@localhost/hexstody
echo "Local database accessible by $DATABASE_URL"
cargo build --release