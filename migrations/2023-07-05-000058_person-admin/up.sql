DROP INDEX IF EXISTS idx_person_admin;

CREATE INDEX idx_person_admin ON person (admin)
WHERE
    admin;

-- allow quickly finding all admins (PersonView::admins)
