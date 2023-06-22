update community c
set local=true
from local_site ls
  join site s on ls.site_id=s.id
where c.instance_id=s.instance_id and not c.local;
