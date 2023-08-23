create database snippetbox; 
create user  boxuser;
grant all privileges on database snippetbox to boxuser;
alter user boxuser  with password 'boxuser-pass';


