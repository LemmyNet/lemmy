-- Change the post url max limit to 2000
-- From here: https://stackoverflow.com/questions/417142/what-is-the-maximum-length-of-a-url-in-different-browsers#417184
ALTER TABLE post
    ALTER COLUMN url TYPE varchar(2000);
