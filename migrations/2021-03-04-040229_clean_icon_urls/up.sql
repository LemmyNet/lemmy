-- If these are not urls, it will crash the server
update user_ set avatar = NULL where avatar not like 'http%';
update user_ set banner = NULL where banner not like 'http%';
update community set icon = NULL where icon not like 'http%';
update community set banner = NULL where banner not like 'http%';
