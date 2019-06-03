-- Host: localhost
-- Database: `mensgleeclub`


CREATE TABLE member (
  email varchar(50) NOT NULL PRIMARY KEY,
  first_name varchar(25) NOT NULL,
  preferred_name varchar(25) DEFAULT NULL,
  last_name varchar(25) NOT NULL,
  pass_hash varchar(64) NOT NULL,
  phone varchar(16) NOT NULL,
  picture varchar(255) DEFAULT NULL,
  passengers int NOT NULL DEFAULT '0',
  location varchar(50) NOT NULL,
  about varchar(500) DEFAULT NULL,
  major varchar(50) DEFAULT NULL,
  minor varchar(50) DEFAULT NULL,
  hometown varchar(50) DEFAULT NULL,
  arrived_at_tech int DEFAULT NULL, -- year
  gateway_drug varchar(500) DEFAULT NULL,
  conflicts varchar(500) DEFAULT NULL,
  dietary_restrictions varchar(500) DEFAULT NULL
) ENGINE=InnoDB DEFAULT CHARSET=utf8;


CREATE TABLE semester (
  id int NOT NULL AUTO_INCREMENT PRIMARY KEY,
  name varchar(32) NOT NULL,
  start_date datetime NOT NULL,
  end_date datetime NOT NULL,
  gig_requirement int NOT NULL DEFAULT '5'
) ENGINE=InnoDB DEFAULT CHARSET=utf8;


CREATE TABLE role (
  id int NOT NULL AUTO_INCREMENT PRIMARY KEY,
  name varchar(20) DEFAULT NULL,
  `rank` int NOT NULL,
  max_quantity int NOT NULL
) ENGINE=InnoDB DEFAULT CHARSET=utf8;


CREATE TABLE member_role (
  member varchar(50) NOT NULL,
  role int NOT NULL,
  semester int NOT NULL,

  PRIMARY KEY (member, role, semester),
  FOREIGN KEY (member) REFERENCES member (email) ON DELETE CASCADE ON UPDATE CASCADE,
  FOREIGN KEY (semester) REFERENCES semester (id) ON DELETE CASCADE ON UPDATE CASCADE,
  FOREIGN KEY (role) REFERENCES role (id) ON DELETE CASCADE ON UPDATE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8;


CREATE TABLE section_type (
  id int NOT NULL AUTO_INCREMENT PRIMARY KEY,
  name varchar(20) NOT NULL
) ENGINE=InnoDB DEFAULT CHARSET=utf8;


CREATE TABLE event_type (
  id int NOT NULL AUTO_INCREMENT PRIMARY KEY,
  name varchar(32) NOT NULL,
  weight int NOT NULL,

  UNIQUE (name)
) ENGINE=InnoDB DEFAULT CHARSET=utf8;


CREATE TABLE event (
  id int NOT NULL AUTO_INCREMENT PRIMARY KEY,
  name varchar(64) NOT NULL,
  semester int NOT NULL,
  `type` int NOT NULL,
  call_time datetime NOT NULL,
  release_time datetime DEFAULT NULL,
  points int NOT NULL,
  comments text DEFAULT NULL,
  location varchar(255) DEFAULT NULL,
  gig_count boolean NOT NULL DEFAULT '1',
  default_attend boolean NOT NULL DEFAULT '1',
  section int DEFAULT NULL,
  
  FOREIGN KEY (semester) REFERENCES semester (id) ON UPDATE CASCADE ON DELETE CASCADE,
  FOREIGN KEY (`type`) REFERENCES event_type (id) ON UPDATE CASCADE ON DELETE CASCADE,
  FOREIGN KEY (section) REFERENCES section_type (id) ON UPDATE CASCADE ON DELETE SET NULL
) ENGINE=InnoDB DEFAULT CHARSET=utf8;


CREATE TABLE absence_request (
  member varchar(50) NOT NULL,
  event int NOT NULL,
  `time` timestamp NOT NULL DEFAULT CURRENT_TIMESTAMP,
  reason varchar(500) NOT NULL,
  state enum('pending', 'approved', 'denied') NOT NULL DEFAULT 'pending',

  PRIMARY KEY (member, event),
  FOREIGN KEY (member) REFERENCES member (email) ON DELETE CASCADE ON UPDATE CASCADE,
  FOREIGN KEY (event) REFERENCES event (id) ON DELETE CASCADE ON UPDATE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8;


CREATE TABLE active_semester (
  member varchar(50) NOT NULL,
  semester int NOT NULL,
  enrollment enum('class', 'club') NOT NULL DEFAULT 'club',
  section int DEFAULT NULL,

  PRIMARY KEY (member, semester),
  FOREIGN KEY (member) REFERENCES member (email) ON DELETE CASCADE ON UPDATE CASCADE,
  FOREIGN KEY (semester) REFERENCES semester (id) ON DELETE CASCADE ON UPDATE CASCADE,
  FOREIGN KEY (section) REFERENCES section_type (id) ON DELETE CASCADE ON UPDATE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8;


CREATE TABLE announcement (
  id int NOT NULL AUTO_INCREMENT PRIMARY KEY,
  member varchar(50) DEFAULT NULL,
  semester int NOT NULL,
  `time` timestamp NOT NULL DEFAULT CURRENT_TIMESTAMP,
  content longtext NOT NULL,
  archived bool NOT NULL DEFAULT '0',

  FOREIGN KEY (member) REFERENCES member (email) ON DELETE SET NULL ON UPDATE CASCADE,
  FOREIGN KEY (semester) REFERENCES semester (id) ON DELETE CASCADE ON UPDATE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8;


CREATE TABLE attendance (
  member varchar(50) NOT NULL,
  event int NOT NULL,
  should_attend boolean NOT NULL DEFAULT '1',
  did_attend boolean DEFAULT NULL DEFAULT '1', -- TODO: null or not if an event hasn't passed
  confirmed boolean NOT NULL DEFAULT '0',
  minutes_late int NOT NULL DEFAULT '0',

  PRIMARY KEY (member, event),
  FOREIGN KEY (member) REFERENCES member (email) ON DELETE CASCADE ON UPDATE CASCADE,
  FOREIGN KEY (event) REFERENCES event (id) ON DELETE CASCADE ON UPDATE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8;


CREATE TABLE carpool (
  id int NOT NULL AUTO_INCREMENT PRIMARY KEY,
  event int NOT NULL,
  driver varchar(50) NOT NULL,

  FOREIGN KEY (event) REFERENCES event (id) ON DELETE CASCADE ON UPDATE CASCADE,
  FOREIGN KEY (driver) REFERENCES member (email) ON DELETE CASCADE ON UPDATE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8;


CREATE TABLE fee (
  name varchar(40) NOT NULL PRIMARY KEY,
  amount int NOT NULL DEFAULT '0'
) ENGINE=InnoDB DEFAULT CHARSET=utf8;


CREATE TABLE google_docs (
  name varchar(40) NOT NULL PRIMARY KEY,
  url varchar(255) NOT NULL
) ENGINE=InnoDB DEFAULT CHARSET=utf8;


CREATE TABLE uniform (
  id int NOT NULL AUTO_INCREMENT PRIMARY KEY,
  name varchar(32) NOT NULL,
  description text DEFAULT NULL
) ENGINE=InnoDB DEFAULT CHARSET=utf8;


CREATE TABLE gig (
  event int NOT NULL PRIMARY KEY,
  performance_time datetime NOT NULL,
  uniform int NOT NULL,
  contact_name varchar(50) DEFAULT NULL,
  contact_email varchar(50) DEFAULT NULL,
  contact_phone varchar(16) DEFAULT NULL,
  price int DEFAULT NULL,
  public boolean NOT NULL DEFAULT '0',
  summary text DEFAULT NULL,
  description text DEFAULT NULL,

  FOREIGN KEY (event) REFERENCES event (id) ON DELETE CASCADE ON UPDATE CASCADE,
  FOREIGN KEY (uniform) REFERENCES uniform (id) ON DELETE CASCADE ON UPDATE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8;


CREATE TABLE gig_request (
  id int NOT NULL AUTO_INCREMENT PRIMARY KEY,
  `time` timestamp NOT NULL DEFAULT CURRENT_TIMESTAMP,
  name varchar(255) NOT NULL,
  organization varchar(255) NOT NULL,
  event int DEFAULT NULL,
  contact_name varchar(255) NOT NULL,
  contact_phone varchar(16) NOT NULL,
  contact_email varchar(50) NOT NULL,
  start_time datetime NOT NULL,
  location varchar(255) NOT NULL,
  comments text DEFAULT NULL,
  status enum('pending', 'accepted', 'dismissed') NOT NULL DEFAULT 'pending',

  FOREIGN KEY (event) REFERENCES event (id) ON UPDATE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8;


CREATE TABLE song (
  id int NOT NULL AUTO_INCREMENT PRIMARY KEY,
  title varchar(128) NOT NULL,
  info text DEFAULT NULL,
  current boolean NOT NULL DEFAULT '0',
  `key` enum('A♭', 'A', 'A#', 'B♭', 'B', 'B#', 'C♭', 'C', 'C♯', 'D♭', 'D', 'D♯', 'E♭',
             'E', 'E#', 'F♭', 'F', 'F♯', 'G♭', 'G', 'G#') DEFAULT NULL,
  starting_pitch enum('A♭', 'A', 'A#', 'B♭', 'B', 'B#', 'C♭', 'C', 'C♯', 'D♭', 'D', 'D♯',
                      'E♭', 'E', 'E#', 'F♭', 'F', 'F♯', 'G♭', 'G', 'G#') DEFAULT NULL,
  mode enum('major', 'minor', 'dorian', 'phrygian', 'lydian',
            'mixolydian', 'locrian') DEFAULT NULL
) ENGINE=InnoDB DEFAULT CHARSET=utf8;


CREATE TABLE gig_song (
  id int NOT NULL AUTO_INCREMENT PRIMARY KEY,
  event int NOT NULL,
  song int NOT NULL,
  `order` int NOT NULL,

  FOREIGN KEY (event) REFERENCES event (id) ON DELETE CASCADE ON UPDATE CASCADE,
  FOREIGN KEY (song) REFERENCES song (id) ON DELETE CASCADE ON UPDATE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8;


CREATE TABLE media_type (
  name varchar(50) NOT NULL PRIMARY KEY,
  `order` int NOT NULL UNIQUE,
  storage enum('local', 'remote') NOT NULL
) ENGINE=InnoDB DEFAULT CHARSET=utf8;


CREATE TABLE minutes (
  id int NOT NULL AUTO_INCREMENT PRIMARY KEY,
  name varchar(100) NOT NULL,
  `date` date NOT NULL,
  private longtext DEFAULT NULL,
  public longtext DEFAULT NULL
) ENGINE=InnoDB DEFAULT CHARSET=utf8;


CREATE TABLE permission (
  name varchar(40) NOT NULL PRIMARY KEY,
  description text DEFAULT NULL,
  `type` enum('static', 'event') NOT NULL DEFAULT 'static'
) ENGINE=InnoDB DEFAULT CHARSET=utf8;


CREATE TABLE rides_in (
  member varchar(50) NOT NULL,
  carpool int NOT NULL,

  PRIMARY KEY (member, carpool),
  FOREIGN KEY (member) REFERENCES member (email) ON DELETE CASCADE ON UPDATE CASCADE,
  FOREIGN KEY (carpool) REFERENCES carpool (id) ON DELETE CASCADE ON UPDATE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8;


CREATE TABLE role_permission (
  id int NOT NULL AUTO_INCREMENT PRIMARY KEY,
  role int NOT NULL,
  permission varchar(40) NOT NULL,
  event_type int DEFAULT NULL,

  FOREIGN KEY (role) REFERENCES role (id) ON DELETE CASCADE ON UPDATE CASCADE,
  FOREIGN KEY (permission) REFERENCES permission (name) ON DELETE CASCADE ON UPDATE CASCADE,
  FOREIGN KEY (event_type) REFERENCES event_type (id) ON DELETE CASCADE ON UPDATE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8;


CREATE TABLE song_link (
  id int NOT NULL AUTO_INCREMENT PRIMARY KEY,
  song int NOT NULL,
  `type` varchar(50) NOT NULL,
  name varchar(128) NOT NULL,
  target varchar(255) NOT NULL,

  FOREIGN KEY (`type`) REFERENCES media_type (name) ON DELETE CASCADE ON UPDATE CASCADE,
  FOREIGN KEY (song) REFERENCES song (id) ON DELETE CASCADE ON UPDATE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8;


CREATE TABLE outfit (
  id int NOT NULL AUTO_INCREMENT PRIMARY KEY,
  name varchar(32) NOT NULL
) ENGINE=InnoDB DEFAULT CHARSET=utf8;


CREATE TABLE outfit_borrow (
  outfit int NOT NULL PRIMARY KEY,
  member varchar(50) NOT NULL,
  `status` enum('circulating', 'lost', 'decommissioned') NOT NULL DEFAULT 'circulating',

  FOREIGN KEY (outfit) REFERENCES outfit (id) ON DELETE CASCADE ON UPDATE CASCADE,
  FOREIGN KEY (member) REFERENCES member (email) ON DELETE CASCADE ON UPDATE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8;


CREATE TABLE todo (
  id int NOT NULL AUTO_INCREMENT PRIMARY KEY,
  `text` varchar(255) NOT NULL,
  member varchar(50) NOT NULL,
  completed boolean NOT NULL DEFAULT '0',

  FOREIGN KEY (member) REFERENCES member (email) ON UPDATE CASCADE ON DELETE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8;


CREATE TABLE transaction_type (
  id int NOT NULL AUTO_INCREMENT PRIMARY KEY,
  name varchar(40) NOT NULL
) ENGINE=InnoDB DEFAULT CHARSET=utf8;


CREATE TABLE transaction (
  id int NOT NULL AUTO_INCREMENT PRIMARY KEY,
  member varchar(50) NOT NULL,
  `time` timestamp NOT NULL DEFAULT CURRENT_TIMESTAMP,
  amount int NOT NULL,
  description varchar(500) NOT NULL,
  semester int DEFAULT NULL,
  `type` int DEFAULT NULL,
  resolved tinyint(1) NOT NULL DEFAULT '0',

  FOREIGN KEY (member) REFERENCES member (email) ON DELETE CASCADE ON UPDATE CASCADE,
  FOREIGN KEY (`type`) REFERENCES transaction_type (id) ON DELETE CASCADE ON UPDATE CASCADE,
  FOREIGN KEY (semester) REFERENCES semester (id) ON UPDATE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8;


CREATE TABLE session (
  member varchar(50) NOT NULL PRIMARY KEY,
  `key` varchar(64) NOT NULL,

  FOREIGN KEY (member) REFERENCES member (email) ON DELETE CASCADE ON UPDATE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8;


CREATE TABLE variable (
  `key` varchar(255) NOT NULL PRIMARY KEY,
  value varchar(255) NOT NULL
) ENGINE=InnoDB DEFAULT CHARSET=utf8;
