CREATE TABLE
LANGUAGE (
    id serial PRIMARY KEY,
    code varchar(3),
    name text
);

CREATE TABLE local_user_language (
    id serial PRIMARY KEY,
    local_user_id int REFERENCES local_user ON UPDATE CASCADE ON DELETE CASCADE NOT NULL,
    language_id int REFERENCES
    LANGUAGE ON
    UPDATE CASCADE ON DELETE CASCADE NOT NULL,
    UNIQUE (local_user_id, language_id)
);

ALTER TABLE local_user RENAME COLUMN lang TO interface_language;

INSERT INTO
LANGUAGE (id, code, name)
    VALUES (0, 'und', 'Undetermined');

ALTER TABLE post
    ADD COLUMN language_id integer REFERENCES LANGUAGE NOT
    NULL DEFAULT 0;

INSERT INTO
LANGUAGE (code, name)
    VALUES ('aa', 'Afaraf');

INSERT INTO
LANGUAGE (code, name)
    VALUES ('ab', 'аҧсуа бызшәа');

INSERT INTO
LANGUAGE (code, name)
    VALUES ('ae', 'avesta');

INSERT INTO
LANGUAGE (code, name)
    VALUES ('af', 'Afrikaans');

INSERT INTO
LANGUAGE (code, name)
    VALUES ('ak', 'Akan');

INSERT INTO
LANGUAGE (code, name)
    VALUES ('am', 'አማርኛ');

INSERT INTO
LANGUAGE (code, name)
    VALUES ('an', 'aragonés');

INSERT INTO
LANGUAGE (code, name)
    VALUES ('ar', 'اَلْعَرَبِيَّةُ');

INSERT INTO
LANGUAGE (code, name)
    VALUES ('as', 'অসমীয়া');

INSERT INTO
LANGUAGE (code, name)
    VALUES ('av', 'авар мацӀ');

INSERT INTO
LANGUAGE (code, name)
    VALUES ('ay', 'aymar aru');

INSERT INTO
LANGUAGE (code, name)
    VALUES ('az', 'azərbaycan dili');

INSERT INTO
LANGUAGE (code, name)
    VALUES ('ba', 'башҡорт теле');

INSERT INTO
LANGUAGE (code, name)
    VALUES ('be', 'беларуская мова');

INSERT INTO
LANGUAGE (code, name)
    VALUES ('bg', 'български език');

INSERT INTO
LANGUAGE (code, name)
    VALUES ('bi', 'Bislama');

INSERT INTO
LANGUAGE (code, name)
    VALUES ('bm', 'bamanankan');

INSERT INTO
LANGUAGE (code, name)
    VALUES ('bn', 'বাংলা');

INSERT INTO
LANGUAGE (code, name)
    VALUES ('bo', 'བོད་ཡིག');

INSERT INTO
LANGUAGE (code, name)
    VALUES ('br', 'brezhoneg');

INSERT INTO
LANGUAGE (code, name)
    VALUES ('bs', 'bosanski jezik');

INSERT INTO
LANGUAGE (code, name)
    VALUES ('ca', 'Català');

INSERT INTO
LANGUAGE (code, name)
    VALUES ('ce', 'нохчийн мотт');

INSERT INTO
LANGUAGE (code, name)
    VALUES ('ch', 'Chamoru');

INSERT INTO
LANGUAGE (code, name)
    VALUES ('co', 'corsu');

INSERT INTO
LANGUAGE (code, name)
    VALUES ('cr', 'ᓀᐦᐃᔭᐍᐏᐣ');

INSERT INTO
LANGUAGE (code, name)
    VALUES ('cs', 'čeština');

INSERT INTO
LANGUAGE (code, name)
    VALUES ('cu', 'ѩзыкъ словѣньскъ');

INSERT INTO
LANGUAGE (code, name)
    VALUES ('cv', 'чӑваш чӗлхи');

INSERT INTO
LANGUAGE (code, name)
    VALUES ('cy', 'Cymraeg');

INSERT INTO
LANGUAGE (code, name)
    VALUES ('da', 'dansk');

INSERT INTO
LANGUAGE (code, name)
    VALUES ('de', 'Deutsch');

INSERT INTO
LANGUAGE (code, name)
    VALUES ('dv', 'ދިވެހި');

INSERT INTO
LANGUAGE (code, name)
    VALUES ('dz', 'རྫོང་ཁ');

INSERT INTO
LANGUAGE (code, name)
    VALUES ('ee', 'Eʋegbe');

INSERT INTO
LANGUAGE (code, name)
    VALUES ('el', 'Ελληνικά');

INSERT INTO
LANGUAGE (code, name)
    VALUES ('en', 'English');

INSERT INTO
LANGUAGE (code, name)
    VALUES ('eo', 'Esperanto');

INSERT INTO
LANGUAGE (code, name)
    VALUES ('es', 'Español');

INSERT INTO
LANGUAGE (code, name)
    VALUES ('et', 'eesti');

INSERT INTO
LANGUAGE (code, name)
    VALUES ('eu', 'euskara');

INSERT INTO
LANGUAGE (code, name)
    VALUES ('fa', 'فارسی');

INSERT INTO
LANGUAGE (code, name)
    VALUES ('ff', 'Fulfulde');

INSERT INTO
LANGUAGE (code, name)
    VALUES ('fi', 'suomi');

INSERT INTO
LANGUAGE (code, name)
    VALUES ('fj', 'vosa Vakaviti');

INSERT INTO
LANGUAGE (code, name)
    VALUES ('fo', 'føroyskt');

INSERT INTO
LANGUAGE (code, name)
    VALUES ('fr', 'Français');

INSERT INTO
LANGUAGE (code, name)
    VALUES ('fy', 'Frysk');

INSERT INTO
LANGUAGE (code, name)
    VALUES ('ga', 'Gaeilge');

INSERT INTO
LANGUAGE (code, name)
    VALUES ('gd', 'Gàidhlig');

INSERT INTO
LANGUAGE (code, name)
    VALUES ('gl', 'galego');

INSERT INTO
LANGUAGE (code, name)
    VALUES ('gn', E'Avañe\'ẽ');

INSERT INTO
LANGUAGE (code, name)
    VALUES ('gu', 'ગુજરાતી');

INSERT INTO
LANGUAGE (code, name)
    VALUES ('gv', 'Gaelg');

INSERT INTO
LANGUAGE (code, name)
    VALUES ('ha', 'هَوُسَ');

INSERT INTO
LANGUAGE (code, name)
    VALUES ('he', 'עברית');

INSERT INTO
LANGUAGE (code, name)
    VALUES ('hi', 'हिन्दी');

INSERT INTO
LANGUAGE (code, name)
    VALUES ('ho', 'Hiri Motu');

INSERT INTO
LANGUAGE (code, name)
    VALUES ('hr', 'Hrvatski');

INSERT INTO
LANGUAGE (code, name)
    VALUES ('ht', 'Kreyòl ayisyen');

INSERT INTO
LANGUAGE (code, name)
    VALUES ('hu', 'magyar');

INSERT INTO
LANGUAGE (code, name)
    VALUES ('hy', 'Հայերեն');

INSERT INTO
LANGUAGE (code, name)
    VALUES ('hz', 'Otjiherero');

INSERT INTO
LANGUAGE (code, name)
    VALUES ('ia', 'Interlingua');

INSERT INTO
LANGUAGE (code, name)
    VALUES ('id', 'Bahasa Indonesia');

INSERT INTO
LANGUAGE (code, name)
    VALUES ('ie', 'Interlingue');

INSERT INTO
LANGUAGE (code, name)
    VALUES ('ig', 'Asụsụ Igbo');

INSERT INTO
LANGUAGE (code, name)
    VALUES ('ii', 'ꆈꌠ꒿ Nuosuhxop');

INSERT INTO
LANGUAGE (code, name)
    VALUES ('ik', 'Iñupiaq');

INSERT INTO
LANGUAGE (code, name)
    VALUES ('io', 'Ido');

INSERT INTO
LANGUAGE (code, name)
    VALUES ('is', 'Íslenska');

INSERT INTO
LANGUAGE (code, name)
    VALUES ('it', 'Italiano');

INSERT INTO
LANGUAGE (code, name)
    VALUES ('iu', 'ᐃᓄᒃᑎᑐᑦ');

INSERT INTO
LANGUAGE (code, name)
    VALUES ('ja', '日本語');

INSERT INTO
LANGUAGE (code, name)
    VALUES ('jv', 'basa Jawa');

INSERT INTO
LANGUAGE (code, name)
    VALUES ('ka', 'ქართული');

INSERT INTO
LANGUAGE (code, name)
    VALUES ('kg', 'Kikongo');

INSERT INTO
LANGUAGE (code, name)
    VALUES ('ki', 'Gĩkũyũ');

INSERT INTO
LANGUAGE (code, name)
    VALUES ('kj', 'Kuanyama');

INSERT INTO
LANGUAGE (code, name)
    VALUES ('kk', 'қазақ тілі');

INSERT INTO
LANGUAGE (code, name)
    VALUES ('kl', 'kalaallisut');

INSERT INTO
LANGUAGE (code, name)
    VALUES ('km', 'ខេមរភាសា');

INSERT INTO
LANGUAGE (code, name)
    VALUES ('kn', 'ಕನ್ನಡ');

INSERT INTO
LANGUAGE (code, name)
    VALUES ('ko', '한국어');

INSERT INTO
LANGUAGE (code, name)
    VALUES ('kr', 'Kanuri');

INSERT INTO
LANGUAGE (code, name)
    VALUES ('ks', 'कश्मीरी');

INSERT INTO
LANGUAGE (code, name)
    VALUES ('ku', 'Kurdî');

INSERT INTO
LANGUAGE (code, name)
    VALUES ('kv', 'коми кыв');

INSERT INTO
LANGUAGE (code, name)
    VALUES ('kw', 'Kernewek');

INSERT INTO
LANGUAGE (code, name)
    VALUES ('ky', 'Кыргызча');

INSERT INTO
LANGUAGE (code, name)
    VALUES ('la', 'latine');

INSERT INTO
LANGUAGE (code, name)
    VALUES ('lb', 'Lëtzebuergesch');

INSERT INTO
LANGUAGE (code, name)
    VALUES ('lg', 'Luganda');

INSERT INTO
LANGUAGE (code, name)
    VALUES ('li', 'Limburgs');

INSERT INTO
LANGUAGE (code, name)
    VALUES ('ln', 'Lingála');

INSERT INTO
LANGUAGE (code, name)
    VALUES ('lo', 'ພາສາລາວ');

INSERT INTO
LANGUAGE (code, name)
    VALUES ('lt', 'lietuvių kalba');

INSERT INTO
LANGUAGE (code, name)
    VALUES ('lu', 'Kiluba');

INSERT INTO
LANGUAGE (code, name)
    VALUES ('lv', 'latviešu valoda');

INSERT INTO
LANGUAGE (code, name)
    VALUES ('mg', 'fiteny malagasy');

INSERT INTO
LANGUAGE (code, name)
    VALUES ('mh', 'Kajin M̧ajeļ');

INSERT INTO
LANGUAGE (code, name)
    VALUES ('mi', 'te reo Māori');

INSERT INTO
LANGUAGE (code, name)
    VALUES ('mk', 'македонски јазик');

INSERT INTO
LANGUAGE (code, name)
    VALUES ('ml', 'മലയാളം');

INSERT INTO
LANGUAGE (code, name)
    VALUES ('mn', 'Монгол хэл');

INSERT INTO
LANGUAGE (code, name)
    VALUES ('mr', 'मराठी');

INSERT INTO
LANGUAGE (code, name)
    VALUES ('ms', 'Bahasa Melayu');

INSERT INTO
LANGUAGE (code, name)
    VALUES ('mt', 'Malti');

INSERT INTO
LANGUAGE (code, name)
    VALUES ('my', 'ဗမာစာ');

INSERT INTO
LANGUAGE (code, name)
    VALUES ('na', 'Dorerin Naoero');

INSERT INTO
LANGUAGE (code, name)
    VALUES ('nb', 'Norsk bokmål');

INSERT INTO
LANGUAGE (code, name)
    VALUES ('nd', 'isiNdebele');

INSERT INTO
LANGUAGE (code, name)
    VALUES ('ne', 'नेपाली');

INSERT INTO
LANGUAGE (code, name)
    VALUES ('ng', 'Owambo');

INSERT INTO
LANGUAGE (code, name)
    VALUES ('nl', 'Nederlands');

INSERT INTO
LANGUAGE (code, name)
    VALUES ('nn', 'Norsk nynorsk');

INSERT INTO
LANGUAGE (code, name)
    VALUES ('no', 'Norsk');

INSERT INTO
LANGUAGE (code, name)
    VALUES ('nr', 'isiNdebele');

INSERT INTO
LANGUAGE (code, name)
    VALUES ('nv', 'Diné bizaad');

INSERT INTO
LANGUAGE (code, name)
    VALUES ('ny', 'chiCheŵa');

INSERT INTO
LANGUAGE (code, name)
    VALUES ('oc', 'occitan');

INSERT INTO
LANGUAGE (code, name)
    VALUES ('oj', 'ᐊᓂᔑᓈᐯᒧᐎᓐ');

INSERT INTO
LANGUAGE (code, name)
    VALUES ('om', 'Afaan Oromoo');

INSERT INTO
LANGUAGE (code, name)
    VALUES ('or', 'ଓଡ଼ିଆ');

INSERT INTO
LANGUAGE (code, name)
    VALUES ('os', 'ирон æвзаг');

INSERT INTO
LANGUAGE (code, name)
    VALUES ('pa', 'ਪੰਜਾਬੀ');

INSERT INTO
LANGUAGE (code, name)
    VALUES ('pi', 'पाऴि');

INSERT INTO
LANGUAGE (code, name)
    VALUES ('pl', 'Polski');

INSERT INTO
LANGUAGE (code, name)
    VALUES ('ps', 'پښتو');

INSERT INTO
LANGUAGE (code, name)
    VALUES ('pt', 'Português');

INSERT INTO
LANGUAGE (code, name)
    VALUES ('qu', 'Runa Simi');

INSERT INTO
LANGUAGE (code, name)
    VALUES ('rm', 'rumantsch grischun');

INSERT INTO
LANGUAGE (code, name)
    VALUES ('rn', 'Ikirundi');

INSERT INTO
LANGUAGE (code, name)
    VALUES ('ro', 'Română');

INSERT INTO
LANGUAGE (code, name)
    VALUES ('ru', 'Русский');

INSERT INTO
LANGUAGE (code, name)
    VALUES ('rw', 'Ikinyarwanda');

INSERT INTO
LANGUAGE (code, name)
    VALUES ('sa', 'संस्कृतम्');

INSERT INTO
LANGUAGE (code, name)
    VALUES ('sc', 'sardu');

INSERT INTO
LANGUAGE (code, name)
    VALUES ('sd', 'सिन्धी');

INSERT INTO
LANGUAGE (code, name)
    VALUES ('se', 'Davvisámegiella');

INSERT INTO
LANGUAGE (code, name)
    VALUES ('sg', 'yângâ tî sängö');

INSERT INTO
LANGUAGE (code, name)
    VALUES ('si', 'සිංහල');

INSERT INTO
LANGUAGE (code, name)
    VALUES ('sk', 'slovenčina');

INSERT INTO
LANGUAGE (code, name)
    VALUES ('sl', 'slovenščina');

INSERT INTO
LANGUAGE (code, name)
    VALUES ('sm', E'gagana fa\'a Samoa');

INSERT INTO
LANGUAGE (code, name)
    VALUES ('sn', 'chiShona');

INSERT INTO
LANGUAGE (code, name)
    VALUES ('so', 'Soomaaliga');

INSERT INTO
LANGUAGE (code, name)
    VALUES ('sq', 'Shqip');

INSERT INTO
LANGUAGE (code, name)
    VALUES ('sr', 'српски језик');

INSERT INTO
LANGUAGE (code, name)
    VALUES ('ss', 'SiSwati');

INSERT INTO
LANGUAGE (code, name)
    VALUES ('st', 'Sesotho');

INSERT INTO
LANGUAGE (code, name)
    VALUES ('su', 'Basa Sunda');

INSERT INTO
LANGUAGE (code, name)
    VALUES ('sv', 'Svenska');

INSERT INTO
LANGUAGE (code, name)
    VALUES ('sw', 'Kiswahili');

INSERT INTO
LANGUAGE (code, name)
    VALUES ('ta', 'தமிழ்');

INSERT INTO
LANGUAGE (code, name)
    VALUES ('te', 'తెలుగు');

INSERT INTO
LANGUAGE (code, name)
    VALUES ('tg', 'тоҷикӣ');

INSERT INTO
LANGUAGE (code, name)
    VALUES ('th', 'ไทย');

INSERT INTO
LANGUAGE (code, name)
    VALUES ('ti', 'ትግርኛ');

INSERT INTO
LANGUAGE (code, name)
    VALUES ('tk', 'Türkmençe');

INSERT INTO
LANGUAGE (code, name)
    VALUES ('tl', 'Wikang Tagalog');

INSERT INTO
LANGUAGE (code, name)
    VALUES ('tn', 'Setswana');

INSERT INTO
LANGUAGE (code, name)
    VALUES ('to', 'faka Tonga');

INSERT INTO
LANGUAGE (code, name)
    VALUES ('tr', 'Türkçe');

INSERT INTO
LANGUAGE (code, name)
    VALUES ('ts', 'Xitsonga');

INSERT INTO
LANGUAGE (code, name)
    VALUES ('tt', 'татар теле');

INSERT INTO
LANGUAGE (code, name)
    VALUES ('tw', 'Twi');

INSERT INTO
LANGUAGE (code, name)
    VALUES ('ty', 'Reo Tahiti');

INSERT INTO
LANGUAGE (code, name)
    VALUES ('ug', 'ئۇيغۇرچە‎');

INSERT INTO
LANGUAGE (code, name)
    VALUES ('uk', 'Українська');

INSERT INTO
LANGUAGE (code, name)
    VALUES ('ur', 'اردو');

INSERT INTO
LANGUAGE (code, name)
    VALUES ('uz', 'Ўзбек');

INSERT INTO
LANGUAGE (code, name)
    VALUES ('ve', 'Tshivenḓa');

INSERT INTO
LANGUAGE (code, name)
    VALUES ('vi', 'Tiếng Việt');

INSERT INTO
LANGUAGE (code, name)
    VALUES ('vo', 'Volapük');

INSERT INTO
LANGUAGE (code, name)
    VALUES ('wa', 'walon');

INSERT INTO
LANGUAGE (code, name)
    VALUES ('wo', 'Wollof');

INSERT INTO
LANGUAGE (code, name)
    VALUES ('xh', 'isiXhosa');

INSERT INTO
LANGUAGE (code, name)
    VALUES ('yi', 'ייִדיש');

INSERT INTO
LANGUAGE (code, name)
    VALUES ('yo', 'Yorùbá');

INSERT INTO
LANGUAGE (code, name)
    VALUES ('za', 'Saɯ cueŋƅ');

INSERT INTO
LANGUAGE (code, name)
    VALUES ('zh', '中文');

INSERT INTO
LANGUAGE (code, name)
    VALUES ('zu', 'isiZulu');

