DO $$
BEGIN
    RAISE 'migrations must be managed using lemmy_server instead of diesel CLI';
END
$$;

