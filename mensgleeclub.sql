-- phpMyAdmin SQL Dump
-- version 4.7.8
-- https://www.phpmyadmin.net/
--
-- Host: localhost:3306
-- Generation Time: Aug 22, 2018 at 03:49 PM
-- Server version: 10.1.35-MariaDB
-- PHP Version: 7.1.14

SET SQL_MODE = "NO_AUTO_VALUE_ON_ZERO";
SET AUTOCOMMIT = 0;
START TRANSACTION;
SET time_zone = "+00:00";


/*!40101 SET @OLD_CHARACTER_SET_CLIENT=@@CHARACTER_SET_CLIENT */;
/*!40101 SET @OLD_CHARACTER_SET_RESULTS=@@CHARACTER_SET_RESULTS */;
/*!40101 SET @OLD_COLLATION_CONNECTION=@@COLLATION_CONNECTION */;
/*!40101 SET NAMES utf8mb4 */;

--
-- Database: `mensgleeclub`
--

-- --------------------------------------------------------

--
-- Table structure for table `absencerequest`
--

CREATE TABLE `absencerequest` (
  `eventNo` int(11) NOT NULL,
  `memberID` varchar(50) NOT NULL,
  `time` timestamp NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  `reason` varchar(500) NOT NULL,
  `replacement` varchar(20) DEFAULT NULL,
  `state` varchar(20) NOT NULL DEFAULT 'pending'
) ENGINE=InnoDB DEFAULT CHARSET=utf8;

-- --------------------------------------------------------

--
-- Table structure for table `activeSemester`
--

CREATE TABLE `activeSemester` (
  `member` varchar(50) NOT NULL,
  `semester` varchar(16) NOT NULL,
  `choir` varchar(16) NOT NULL,
  `enrollment` enum('class','club') NOT NULL DEFAULT 'club',
  `section` int(1) NOT NULL DEFAULT '0'
) ENGINE=InnoDB DEFAULT CHARSET=utf8;

-- --------------------------------------------------------

--
-- Table structure for table `announcement`
--

CREATE TABLE `announcement` (
  `announcementNo` int(11) NOT NULL,
  `choir` varchar(16) NOT NULL,
  `memberID` varchar(50) NOT NULL,
  `timePosted` timestamp NOT NULL DEFAULT CURRENT_TIMESTAMP,
  `announcement` longtext NOT NULL,
  `archived` tinyint(1) NOT NULL DEFAULT '0'
) ENGINE=InnoDB DEFAULT CHARSET=utf8;

-- --------------------------------------------------------

--
-- Table structure for table `attends`
--

CREATE TABLE `attends` (
  `memberID` varchar(50) NOT NULL,
  `shouldAttend` tinyint(1) NOT NULL DEFAULT '1',
  `didAttend` tinyint(1) DEFAULT NULL,
  `eventNo` int(11) NOT NULL DEFAULT '0',
  `minutesLate` int(11) NOT NULL DEFAULT '0',
  `confirmed` tinyint(1) NOT NULL DEFAULT '0'
) ENGINE=InnoDB DEFAULT CHARSET=utf8;

-- --------------------------------------------------------

--
-- Table structure for table `carpool`
--

CREATE TABLE `carpool` (
  `carpoolID` int(11) NOT NULL,
  `driver` varchar(50) NOT NULL,
  `eventNo` int(11) NOT NULL
) ENGINE=InnoDB DEFAULT CHARSET=utf8;

-- --------------------------------------------------------

--
-- Table structure for table `choir`
--

CREATE TABLE `choir` (
  `id` varchar(16) NOT NULL,
  `name` varchar(64) NOT NULL,
  `admin` varchar(128) NOT NULL,
  `list` varchar(128) NOT NULL
) ENGINE=InnoDB DEFAULT CHARSET=utf8;

-- --------------------------------------------------------

--
-- Table structure for table `event`
--

CREATE TABLE `event` (
  `eventNo` int(11) NOT NULL,
  `name` varchar(50) NOT NULL,
  `choir` varchar(16) NOT NULL,
  `callTime` datetime NOT NULL,
  `releaseTime` datetime DEFAULT NULL,
  `points` int(2) NOT NULL,
  `comments` varchar(1500) DEFAULT NULL,
  `type` varchar(16) NOT NULL,
  `location` varchar(500) DEFAULT NULL,
  `semester` varchar(16) NOT NULL,
  `gigcount` tinyint(1) NOT NULL DEFAULT '1',
  `section` int(1) NOT NULL DEFAULT '0',
  `defaultAttend` tinyint(1) NOT NULL DEFAULT '1'
) ENGINE=InnoDB DEFAULT CHARSET=utf8;

-- --------------------------------------------------------

--
-- Table structure for table `eventType`
--

CREATE TABLE `eventType` (
  `id` varchar(16) NOT NULL,
  `name` varchar(64) NOT NULL,
  `weight` int(11) NOT NULL
) ENGINE=InnoDB DEFAULT CHARSET=utf8;

-- --------------------------------------------------------

--
-- Table structure for table `fee`
--

CREATE TABLE `fee` (
  `id` varchar(16) NOT NULL,
  `name` varchar(40) DEFAULT NULL,
  `choir` varchar(16) NOT NULL,
  `amount` int(11) NOT NULL DEFAULT '0'
) ENGINE=InnoDB DEFAULT CHARSET=utf8;

-- --------------------------------------------------------

--
-- Table structure for table `gdocs`
--

CREATE TABLE `gdocs` (
  `name` varchar(40) NOT NULL,
  `choir` varchar(16) NOT NULL,
  `url` varchar(128) NOT NULL
) ENGINE=InnoDB DEFAULT CHARSET=utf8;

-- --------------------------------------------------------

--
-- Table structure for table `gig`
--

CREATE TABLE `gig` (
  `eventNo` int(11) NOT NULL,
  `performanceTime` datetime NOT NULL,
  `uniform` varchar(13) NOT NULL,
  `cname` varchar(20) DEFAULT NULL,
  `cemail` varchar(50) DEFAULT NULL,
  `cphone` varchar(16) DEFAULT NULL,
  `price` int(4) DEFAULT NULL,
  `public` tinyint(1) NOT NULL DEFAULT '0',
  `summary` text NOT NULL,
  `description` text NOT NULL
) ENGINE=InnoDB DEFAULT CHARSET=utf8;

-- --------------------------------------------------------

--
-- Table structure for table `gigreq`
--

CREATE TABLE `gigreq` (
  `id` int(11) NOT NULL,
  `timestamp` timestamp NOT NULL DEFAULT CURRENT_TIMESTAMP,
  `name` varchar(256) NOT NULL,
  `org` varchar(256) NOT NULL,
  `eventNo` int(11) DEFAULT NULL,
  `cname` varchar(256) NOT NULL,
  `cphone` varchar(10) NOT NULL,
  `cemail` varchar(256) NOT NULL,
  `startTime` datetime NOT NULL,
  `location` varchar(512) NOT NULL,
  `comments` text NOT NULL,
  `status` enum('pending','accepted','dismissed') NOT NULL DEFAULT 'pending',
  `semester` varchar(16) NOT NULL
) ENGINE=InnoDB DEFAULT CHARSET=utf8;

-- --------------------------------------------------------

--
-- Table structure for table `gigSong`
--

CREATE TABLE `gigSong` (
  `id` int(11) NOT NULL,
  `event` int(11) NOT NULL,
  `song` int(11) NOT NULL,
  `order` int(11) NOT NULL
) ENGINE=InnoDB DEFAULT CHARSET=latin1;

-- --------------------------------------------------------

--
-- Table structure for table `mediaType`
--

CREATE TABLE `mediaType` (
  `typeid` varchar(10) NOT NULL,
  `order` smallint(6) NOT NULL,
  `name` varchar(128) NOT NULL,
  `storage` enum('local','remote') NOT NULL
) ENGINE=InnoDB DEFAULT CHARSET=utf8;

-- --------------------------------------------------------

--
-- Table structure for table `member`
--

CREATE TABLE `member` (
  `firstName` varchar(20) DEFAULT NULL,
  `prefName` varchar(20) DEFAULT NULL,
  `lastName` varchar(20) DEFAULT NULL,
  `email` varchar(50) NOT NULL DEFAULT '',
  `password` varchar(50) DEFAULT NULL,
  `phone` bigint(10) DEFAULT NULL,
  `picture` varchar(100) DEFAULT NULL,
  `passengers` int(3) NOT NULL DEFAULT '0',
  `onCampus` tinyint(1) DEFAULT NULL,
  `location` varchar(50) DEFAULT NULL,
  `about` varchar(500) DEFAULT NULL,
  `major` varchar(50) DEFAULT NULL,
  `minor` varchar(50) DEFAULT NULL,
  `hometown` varchar(50) DEFAULT NULL,
  `techYear` int(1) DEFAULT NULL,
  `gChat` varchar(20) DEFAULT NULL,
  `twitter` varchar(20) DEFAULT NULL,
  `gatewayDrug` varchar(500) DEFAULT NULL,
  `conflicts` varchar(500) DEFAULT NULL
) ENGINE=InnoDB DEFAULT CHARSET=utf8;

-- --------------------------------------------------------

--
-- Table structure for table `memberRole`
--

CREATE TABLE `memberRole` (
  `member` varchar(50) NOT NULL,
  `role` int(11) NOT NULL,
  `semester` varchar(16) NOT NULL
) ENGINE=InnoDB DEFAULT CHARSET=utf8;

-- --------------------------------------------------------

--
-- Table structure for table `minutes`
--

CREATE TABLE `minutes` (
  `id` int(11) NOT NULL,
  `choir` varchar(16) NOT NULL,
  `date` date NOT NULL,
  `name` varchar(100) NOT NULL,
  `private` longtext,
  `public` longtext
) ENGINE=InnoDB DEFAULT CHARSET=utf8;

-- --------------------------------------------------------

--
-- Table structure for table `permission`
--

CREATE TABLE `permission` (
  `name` varchar(40) NOT NULL,
  `description` text,
  `type` enum('static','event') NOT NULL DEFAULT 'static'
) ENGINE=InnoDB DEFAULT CHARSET=utf8;

-- --------------------------------------------------------

--
-- Table structure for table `requestState`
--

CREATE TABLE `requestState` (
  `stateName` varchar(20) NOT NULL
) ENGINE=InnoDB DEFAULT CHARSET=utf8;

-- --------------------------------------------------------

--
-- Table structure for table `ridesin`
--

CREATE TABLE `ridesin` (
  `memberID` varchar(50) NOT NULL,
  `carpoolID` int(11) NOT NULL
) ENGINE=InnoDB DEFAULT CHARSET=utf8;

-- --------------------------------------------------------

--
-- Table structure for table `role`
--

CREATE TABLE `role` (
  `id` int(1) NOT NULL,
  `name` varchar(20) DEFAULT NULL,
  `choir` varchar(16) NOT NULL,
  `rank` int(11) NOT NULL,
  `quantity` int(11) NOT NULL
) ENGINE=InnoDB DEFAULT CHARSET=utf8;

-- --------------------------------------------------------

--
-- Table structure for table `rolePermission`
--

CREATE TABLE `rolePermission` (
  `id` int(11) NOT NULL,
  `role` int(1) NOT NULL,
  `permission` varchar(40) NOT NULL,
  `eventType` varchar(16) DEFAULT NULL
) ENGINE=InnoDB DEFAULT CHARSET=utf8;

-- --------------------------------------------------------

--
-- Table structure for table `sectionType`
--

CREATE TABLE `sectionType` (
  `id` int(1) NOT NULL,
  `name` varchar(20) DEFAULT NULL,
  `choir` varchar(16) DEFAULT NULL
) ENGINE=InnoDB DEFAULT CHARSET=utf8;

-- --------------------------------------------------------

--
-- Table structure for table `semester`
--

CREATE TABLE `semester` (
  `semester` varchar(16) NOT NULL,
  `beginning` datetime NOT NULL,
  `end` datetime NOT NULL,
  `gigreq` int(11) NOT NULL DEFAULT '5'
) ENGINE=InnoDB DEFAULT CHARSET=utf8;

-- --------------------------------------------------------

--
-- Table structure for table `song`
--

CREATE TABLE `song` (
  `id` int(11) NOT NULL,
  `choir` varchar(16) NOT NULL,
  `title` varchar(128) NOT NULL,
  `info` text NOT NULL,
  `current` tinyint(1) NOT NULL DEFAULT '0',
  `key` enum('?','A♭','a♭','A','a','a♯','B♭','b♭','B','b','C♭','C','c','C♯','c♯','D♭','D','d','d♯','E♭','e♭','E','e','F','f','F♯','f♯','G♭','G','g','g♯') CHARACTER SET utf8 COLLATE utf8_bin NOT NULL DEFAULT '?',
  `pitch` enum('?','A♭','A','A♯','B♭','B','C','C♯','D♭','D','D♯','E♭','E','F','F♯','G♭','G','G♯') CHARACTER SET utf8 COLLATE utf8_bin NOT NULL DEFAULT '?'
) ENGINE=InnoDB DEFAULT CHARSET=utf8;

-- --------------------------------------------------------

--
-- Table structure for table `songLink`
--

CREATE TABLE `songLink` (
  `id` int(11) NOT NULL,
  `type` varchar(16) NOT NULL,
  `name` varchar(128) NOT NULL,
  `target` varchar(128) NOT NULL,
  `song` int(11) NOT NULL
) ENGINE=InnoDB DEFAULT CHARSET=utf8;

-- --------------------------------------------------------

--
-- Table structure for table `tie`
--

CREATE TABLE `tie` (
  `id` int(11) NOT NULL DEFAULT '0',
  `status` varchar(20) CHARACTER SET utf8 NOT NULL DEFAULT 'Circulating',
  `comments` varchar(200) CHARACTER SET utf8 DEFAULT NULL
) ENGINE=InnoDB DEFAULT CHARSET=latin1;

-- --------------------------------------------------------

--
-- Table structure for table `tieBorrow`
--

CREATE TABLE `tieBorrow` (
  `id` int(11) NOT NULL,
  `member` varchar(50) NOT NULL,
  `tie` int(11) NOT NULL,
  `dateOut` date NOT NULL,
  `dateIn` date DEFAULT NULL
) ENGINE=InnoDB DEFAULT CHARSET=utf8;

-- --------------------------------------------------------

--
-- Table structure for table `tieStatus`
--

CREATE TABLE `tieStatus` (
  `name` varchar(20) CHARACTER SET utf8 NOT NULL
) ENGINE=InnoDB DEFAULT CHARSET=latin1;

-- --------------------------------------------------------

--
-- Table structure for table `todo`
--

CREATE TABLE `todo` (
  `id` int(11) NOT NULL,
  `text` text NOT NULL,
  `completed` tinyint(1) NOT NULL
) ENGINE=InnoDB DEFAULT CHARSET=utf8;

-- --------------------------------------------------------

--
-- Table structure for table `todoMembers`
--

CREATE TABLE `todoMembers` (
  `memberID` varchar(50) NOT NULL,
  `todoID` int(11) NOT NULL
) ENGINE=InnoDB DEFAULT CHARSET=utf8;

-- --------------------------------------------------------

--
-- Table structure for table `transaction`
--

CREATE TABLE `transaction` (
  `memberID` varchar(50) NOT NULL,
  `choir` varchar(16) NOT NULL,
  `time` timestamp NOT NULL DEFAULT CURRENT_TIMESTAMP,
  `amount` int(4) NOT NULL,
  `description` varchar(500) NOT NULL,
  `transactionID` int(11) NOT NULL,
  `semester` varchar(16) DEFAULT NULL,
  `type` varchar(20) NOT NULL DEFAULT 'deposit',
  `resolved` tinyint(1) NOT NULL DEFAULT '0'
) ENGINE=InnoDB DEFAULT CHARSET=utf8;

-- --------------------------------------------------------

--
-- Table structure for table `transacType`
--

CREATE TABLE `transacType` (
  `id` varchar(20) CHARACTER SET utf8 NOT NULL,
  `name` varchar(40) CHARACTER SET utf8 NOT NULL
) ENGINE=InnoDB DEFAULT CHARSET=latin1;

-- --------------------------------------------------------

--
-- Table structure for table `uniform`
--

CREATE TABLE `uniform` (
  `id` varchar(20) NOT NULL,
  `choir` varchar(16) NOT NULL DEFAULT '',
  `name` varchar(20) NOT NULL,
  `description` text NOT NULL
) ENGINE=InnoDB DEFAULT CHARSET=utf8;

-- --------------------------------------------------------

--
-- Table structure for table `variables`
--

CREATE TABLE `variables` (
  `semester` varchar(16) NOT NULL,
  `duesAmount` int(11) NOT NULL,
  `tieDeposit` int(11) NOT NULL,
  `lateFee` int(11) NOT NULL,
  `gigCheck` tinyint(1) NOT NULL DEFAULT '0'
) ENGINE=InnoDB DEFAULT CHARSET=utf8;

--
-- Indexes for dumped tables
--

--
-- Indexes for table `absencerequest`
--
ALTER TABLE `absencerequest`
  ADD PRIMARY KEY (`memberID`,`eventNo`),
  ADD KEY `eventNo` (`eventNo`),
  ADD KEY `state` (`state`);

--
-- Indexes for table `activeSemester`
--
ALTER TABLE `activeSemester`
  ADD PRIMARY KEY (`member`,`semester`,`choir`),
  ADD KEY `member` (`member`),
  ADD KEY `semester` (`semester`),
  ADD KEY `choir` (`choir`),
  ADD KEY `section` (`section`);

--
-- Indexes for table `announcement`
--
ALTER TABLE `announcement`
  ADD PRIMARY KEY (`announcementNo`),
  ADD KEY `memberID` (`memberID`),
  ADD KEY `choir` (`choir`);

--
-- Indexes for table `attends`
--
ALTER TABLE `attends`
  ADD PRIMARY KEY (`memberID`,`eventNo`),
  ADD KEY `eventNo` (`eventNo`);

--
-- Indexes for table `carpool`
--
ALTER TABLE `carpool`
  ADD PRIMARY KEY (`carpoolID`),
  ADD KEY `eventNo` (`eventNo`),
  ADD KEY `driver_memberID` (`driver`);

--
-- Indexes for table `choir`
--
ALTER TABLE `choir`
  ADD PRIMARY KEY (`id`);

--
-- Indexes for table `event`
--
ALTER TABLE `event`
  ADD PRIMARY KEY (`eventNo`),
  ADD KEY `event_validSemester` (`semester`),
  ADD KEY `section` (`section`),
  ADD KEY `choir` (`choir`),
  ADD KEY `type_2` (`type`);

--
-- Indexes for table `eventType`
--
ALTER TABLE `eventType`
  ADD PRIMARY KEY (`id`),
  ADD UNIQUE KEY `id` (`id`);

--
-- Indexes for table `fee`
--
ALTER TABLE `fee`
  ADD PRIMARY KEY (`id`),
  ADD KEY `choir` (`choir`);

--
-- Indexes for table `gdocs`
--
ALTER TABLE `gdocs`
  ADD PRIMARY KEY (`name`,`choir`),
  ADD KEY `choir` (`choir`);

--
-- Indexes for table `gig`
--
ALTER TABLE `gig`
  ADD PRIMARY KEY (`eventNo`),
  ADD KEY `uniform` (`uniform`);

--
-- Indexes for table `gigreq`
--
ALTER TABLE `gigreq`
  ADD PRIMARY KEY (`id`),
  ADD KEY `semester` (`semester`),
  ADD KEY `eventNo` (`eventNo`);

--
-- Indexes for table `gigSong`
--
ALTER TABLE `gigSong`
  ADD PRIMARY KEY (`id`),
  ADD KEY `event` (`event`),
  ADD KEY `song` (`song`);

--
-- Indexes for table `mediaType`
--
ALTER TABLE `mediaType`
  ADD PRIMARY KEY (`typeid`);

--
-- Indexes for table `member`
--
ALTER TABLE `member`
  ADD PRIMARY KEY (`email`);

--
-- Indexes for table `memberRole`
--
ALTER TABLE `memberRole`
  ADD PRIMARY KEY (`member`,`role`,`semester`),
  ADD KEY `member` (`member`,`role`,`semester`),
  ADD KEY `semester` (`semester`),
  ADD KEY `role` (`role`);

--
-- Indexes for table `minutes`
--
ALTER TABLE `minutes`
  ADD PRIMARY KEY (`id`),
  ADD KEY `choir` (`choir`);

--
-- Indexes for table `permission`
--
ALTER TABLE `permission`
  ADD PRIMARY KEY (`name`);

--
-- Indexes for table `requestState`
--
ALTER TABLE `requestState`
  ADD PRIMARY KEY (`stateName`);

--
-- Indexes for table `ridesin`
--
ALTER TABLE `ridesin`
  ADD PRIMARY KEY (`memberID`,`carpoolID`),
  ADD KEY `carpoolID` (`carpoolID`);

--
-- Indexes for table `role`
--
ALTER TABLE `role`
  ADD PRIMARY KEY (`id`),
  ADD UNIQUE KEY `typeName` (`name`),
  ADD KEY `choir` (`choir`);

--
-- Indexes for table `rolePermission`
--
ALTER TABLE `rolePermission`
  ADD PRIMARY KEY (`id`),
  ADD KEY `role` (`role`),
  ADD KEY `permission` (`permission`),
  ADD KEY `eventType` (`eventType`);

--
-- Indexes for table `sectionType`
--
ALTER TABLE `sectionType`
  ADD PRIMARY KEY (`id`),
  ADD KEY `choir` (`choir`);

--
-- Indexes for table `semester`
--
ALTER TABLE `semester`
  ADD PRIMARY KEY (`semester`);

--
-- Indexes for table `song`
--
ALTER TABLE `song`
  ADD PRIMARY KEY (`id`),
  ADD KEY `choir` (`choir`);

--
-- Indexes for table `songLink`
--
ALTER TABLE `songLink`
  ADD PRIMARY KEY (`id`),
  ADD KEY `type` (`type`),
  ADD KEY `song` (`song`);

--
-- Indexes for table `tie`
--
ALTER TABLE `tie`
  ADD PRIMARY KEY (`id`),
  ADD KEY `status` (`status`);

--
-- Indexes for table `tieBorrow`
--
ALTER TABLE `tieBorrow`
  ADD PRIMARY KEY (`id`),
  ADD KEY `member` (`member`),
  ADD KEY `tie` (`tie`);

--
-- Indexes for table `tieStatus`
--
ALTER TABLE `tieStatus`
  ADD PRIMARY KEY (`name`);

--
-- Indexes for table `todo`
--
ALTER TABLE `todo`
  ADD PRIMARY KEY (`id`);

--
-- Indexes for table `todoMembers`
--
ALTER TABLE `todoMembers`
  ADD KEY `memberID` (`memberID`),
  ADD KEY `todoID` (`todoID`);

--
-- Indexes for table `transaction`
--
ALTER TABLE `transaction`
  ADD PRIMARY KEY (`transactionID`),
  ADD KEY `memberID` (`memberID`),
  ADD KEY `type` (`type`),
  ADD KEY `semester` (`semester`),
  ADD KEY `choir` (`choir`);

--
-- Indexes for table `transacType`
--
ALTER TABLE `transacType`
  ADD PRIMARY KEY (`id`);

--
-- Indexes for table `uniform`
--
ALTER TABLE `uniform`
  ADD PRIMARY KEY (`id`,`choir`),
  ADD KEY `choir` (`choir`);

--
-- Indexes for table `variables`
--
ALTER TABLE `variables`
  ADD KEY `variable_validSemester` (`semester`);

--
-- AUTO_INCREMENT for dumped tables
--

--
-- AUTO_INCREMENT for table `announcement`
--
ALTER TABLE `announcement`
  MODIFY `announcementNo` int(11) NOT NULL AUTO_INCREMENT, AUTO_INCREMENT=78;

--
-- AUTO_INCREMENT for table `carpool`
--
ALTER TABLE `carpool`
  MODIFY `carpoolID` int(11) NOT NULL AUTO_INCREMENT, AUTO_INCREMENT=595;

--
-- AUTO_INCREMENT for table `event`
--
ALTER TABLE `event`
  MODIFY `eventNo` int(11) NOT NULL AUTO_INCREMENT, AUTO_INCREMENT=2338;

--
-- AUTO_INCREMENT for table `gigreq`
--
ALTER TABLE `gigreq`
  MODIFY `id` int(11) NOT NULL AUTO_INCREMENT, AUTO_INCREMENT=14;

--
-- AUTO_INCREMENT for table `gigSong`
--
ALTER TABLE `gigSong`
  MODIFY `id` int(11) NOT NULL AUTO_INCREMENT, AUTO_INCREMENT=105;

--
-- AUTO_INCREMENT for table `minutes`
--
ALTER TABLE `minutes`
  MODIFY `id` int(11) NOT NULL AUTO_INCREMENT, AUTO_INCREMENT=281;

--
-- AUTO_INCREMENT for table `role`
--
ALTER TABLE `role`
  MODIFY `id` int(1) NOT NULL AUTO_INCREMENT, AUTO_INCREMENT=15;

--
-- AUTO_INCREMENT for table `rolePermission`
--
ALTER TABLE `rolePermission`
  MODIFY `id` int(11) NOT NULL AUTO_INCREMENT, AUTO_INCREMENT=167;

--
-- AUTO_INCREMENT for table `sectionType`
--
ALTER TABLE `sectionType`
  MODIFY `id` int(1) NOT NULL AUTO_INCREMENT, AUTO_INCREMENT=31;

--
-- AUTO_INCREMENT for table `song`
--
ALTER TABLE `song`
  MODIFY `id` int(11) NOT NULL AUTO_INCREMENT, AUTO_INCREMENT=144;

--
-- AUTO_INCREMENT for table `songLink`
--
ALTER TABLE `songLink`
  MODIFY `id` int(11) NOT NULL AUTO_INCREMENT, AUTO_INCREMENT=691;

--
-- AUTO_INCREMENT for table `tieBorrow`
--
ALTER TABLE `tieBorrow`
  MODIFY `id` int(11) NOT NULL AUTO_INCREMENT, AUTO_INCREMENT=48;

--
-- AUTO_INCREMENT for table `todo`
--
ALTER TABLE `todo`
  MODIFY `id` int(11) NOT NULL AUTO_INCREMENT, AUTO_INCREMENT=104;

--
-- AUTO_INCREMENT for table `transaction`
--
ALTER TABLE `transaction`
  MODIFY `transactionID` int(11) NOT NULL AUTO_INCREMENT, AUTO_INCREMENT=1199;

--
-- Constraints for dumped tables
--

--
-- Constraints for table `absencerequest`
--
ALTER TABLE `absencerequest`
  ADD CONSTRAINT `absencerequest_ibfk_1` FOREIGN KEY (`memberID`) REFERENCES `member` (`email`) ON DELETE CASCADE ON UPDATE CASCADE,
  ADD CONSTRAINT `absencerequest_ibfk_2` FOREIGN KEY (`eventNo`) REFERENCES `event` (`eventNo`) ON DELETE CASCADE ON UPDATE CASCADE,
  ADD CONSTRAINT `absencerequest_ibfk_3` FOREIGN KEY (`state`) REFERENCES `requestState` (`stateName`);

--
-- Constraints for table `activeSemester`
--
ALTER TABLE `activeSemester`
  ADD CONSTRAINT `activeSemester_ibfk_1` FOREIGN KEY (`member`) REFERENCES `member` (`email`) ON DELETE CASCADE ON UPDATE CASCADE,
  ADD CONSTRAINT `activeSemester_ibfk_2` FOREIGN KEY (`semester`) REFERENCES `semester` (`semester`) ON DELETE CASCADE ON UPDATE CASCADE,
  ADD CONSTRAINT `activeSemester_ibfk_3` FOREIGN KEY (`choir`) REFERENCES `choir` (`id`),
  ADD CONSTRAINT `activeSemester_ibfk_4` FOREIGN KEY (`section`) REFERENCES `sectionType` (`id`) ON UPDATE CASCADE;

--
-- Constraints for table `announcement`
--
ALTER TABLE `announcement`
  ADD CONSTRAINT `announcement_ibfk_1` FOREIGN KEY (`memberID`) REFERENCES `member` (`email`) ON DELETE CASCADE ON UPDATE CASCADE,
  ADD CONSTRAINT `announcement_ibfk_2` FOREIGN KEY (`choir`) REFERENCES `choir` (`id`) ON DELETE CASCADE ON UPDATE CASCADE;

--
-- Constraints for table `attends`
--
ALTER TABLE `attends`
  ADD CONSTRAINT `attends_ibfk_2` FOREIGN KEY (`memberID`) REFERENCES `member` (`email`) ON DELETE CASCADE ON UPDATE CASCADE,
  ADD CONSTRAINT `attends_ibfk_3` FOREIGN KEY (`eventNo`) REFERENCES `event` (`eventNo`) ON DELETE CASCADE ON UPDATE CASCADE;

--
-- Constraints for table `carpool`
--
ALTER TABLE `carpool`
  ADD CONSTRAINT `carpool_ibfk_1` FOREIGN KEY (`eventNo`) REFERENCES `event` (`eventNo`) ON DELETE CASCADE ON UPDATE CASCADE,
  ADD CONSTRAINT `carpool_ibfk_2` FOREIGN KEY (`driver`) REFERENCES `member` (`email`) ON DELETE CASCADE ON UPDATE CASCADE;

--
-- Constraints for table `event`
--
ALTER TABLE `event`
  ADD CONSTRAINT `event_ibfk_3` FOREIGN KEY (`choir`) REFERENCES `choir` (`id`) ON UPDATE CASCADE,
  ADD CONSTRAINT `event_ibfk_4` FOREIGN KEY (`type`) REFERENCES `eventType` (`id`),
  ADD CONSTRAINT `event_ibfk_5` FOREIGN KEY (`section`) REFERENCES `sectionType` (`id`) ON UPDATE CASCADE,
  ADD CONSTRAINT `event_validSemester` FOREIGN KEY (`semester`) REFERENCES `semester` (`semester`) ON DELETE CASCADE ON UPDATE CASCADE;

--
-- Constraints for table `fee`
--
ALTER TABLE `fee`
  ADD CONSTRAINT `fee_ibfk_1` FOREIGN KEY (`choir`) REFERENCES `choir` (`id`) ON DELETE CASCADE ON UPDATE CASCADE;

--
-- Constraints for table `gdocs`
--
ALTER TABLE `gdocs`
  ADD CONSTRAINT `gdocs_ibfk_1` FOREIGN KEY (`choir`) REFERENCES `choir` (`id`) ON DELETE CASCADE ON UPDATE CASCADE;

--
-- Constraints for table `gig`
--
ALTER TABLE `gig`
  ADD CONSTRAINT `gig_ibfk_1` FOREIGN KEY (`eventNo`) REFERENCES `event` (`eventNo`) ON DELETE CASCADE ON UPDATE CASCADE,
  ADD CONSTRAINT `gig_ibfk_2` FOREIGN KEY (`uniform`) REFERENCES `uniform` (`id`);

--
-- Constraints for table `gigreq`
--
ALTER TABLE `gigreq`
  ADD CONSTRAINT `gigreq_ibfk_1` FOREIGN KEY (`semester`) REFERENCES `semester` (`semester`),
  ADD CONSTRAINT `gigreq_ibfk_2` FOREIGN KEY (`eventNo`) REFERENCES `event` (`eventNo`) ON DELETE SET NULL ON UPDATE CASCADE;

--
-- Constraints for table `gigSong`
--
ALTER TABLE `gigSong`
  ADD CONSTRAINT `gigSong_ibfk_1` FOREIGN KEY (`event`) REFERENCES `event` (`eventNo`) ON DELETE CASCADE ON UPDATE CASCADE,
  ADD CONSTRAINT `gigSong_ibfk_2` FOREIGN KEY (`song`) REFERENCES `song` (`id`) ON DELETE CASCADE ON UPDATE CASCADE;

--
-- Constraints for table `memberRole`
--
ALTER TABLE `memberRole`
  ADD CONSTRAINT `memberRole_ibfk_1` FOREIGN KEY (`member`) REFERENCES `member` (`email`) ON DELETE CASCADE ON UPDATE CASCADE,
  ADD CONSTRAINT `memberRole_ibfk_3` FOREIGN KEY (`semester`) REFERENCES `semester` (`semester`) ON DELETE CASCADE ON UPDATE CASCADE,
  ADD CONSTRAINT `memberRole_ibfk_4` FOREIGN KEY (`role`) REFERENCES `role` (`id`) ON DELETE CASCADE ON UPDATE CASCADE;

--
-- Constraints for table `minutes`
--
ALTER TABLE `minutes`
  ADD CONSTRAINT `minutes_ibfk_1` FOREIGN KEY (`choir`) REFERENCES `choir` (`id`);

--
-- Constraints for table `ridesin`
--
ALTER TABLE `ridesin`
  ADD CONSTRAINT `ridesin_ibfk_1` FOREIGN KEY (`memberID`) REFERENCES `member` (`email`) ON DELETE CASCADE ON UPDATE CASCADE,
  ADD CONSTRAINT `ridesin_ibfk_2` FOREIGN KEY (`carpoolID`) REFERENCES `carpool` (`carpoolID`) ON DELETE CASCADE ON UPDATE CASCADE;

--
-- Constraints for table `role`
--
ALTER TABLE `role`
  ADD CONSTRAINT `role_ibfk_1` FOREIGN KEY (`choir`) REFERENCES `choir` (`id`) ON DELETE CASCADE ON UPDATE CASCADE;

--
-- Constraints for table `rolePermission`
--
ALTER TABLE `rolePermission`
  ADD CONSTRAINT `rolePermission_ibfk_1` FOREIGN KEY (`role`) REFERENCES `role` (`id`) ON DELETE CASCADE ON UPDATE CASCADE,
  ADD CONSTRAINT `rolePermission_ibfk_2` FOREIGN KEY (`permission`) REFERENCES `permission` (`name`) ON DELETE CASCADE ON UPDATE CASCADE,
  ADD CONSTRAINT `rolePermission_ibfk_3` FOREIGN KEY (`eventType`) REFERENCES `eventType` (`id`) ON DELETE CASCADE ON UPDATE CASCADE;

--
-- Constraints for table `sectionType`
--
ALTER TABLE `sectionType`
  ADD CONSTRAINT `sectionType_ibfk_1` FOREIGN KEY (`choir`) REFERENCES `choir` (`id`) ON UPDATE CASCADE;

--
-- Constraints for table `song`
--
ALTER TABLE `song`
  ADD CONSTRAINT `song_ibfk_1` FOREIGN KEY (`choir`) REFERENCES `choir` (`id`);

--
-- Constraints for table `songLink`
--
ALTER TABLE `songLink`
  ADD CONSTRAINT `songLink_ibfk_1` FOREIGN KEY (`type`) REFERENCES `mediaType` (`typeid`) ON DELETE CASCADE ON UPDATE CASCADE,
  ADD CONSTRAINT `songLink_ibfk_2` FOREIGN KEY (`song`) REFERENCES `song` (`id`) ON DELETE CASCADE ON UPDATE CASCADE;

--
-- Constraints for table `tie`
--
ALTER TABLE `tie`
  ADD CONSTRAINT `tie_ibfk_1` FOREIGN KEY (`status`) REFERENCES `tieStatus` (`name`) ON UPDATE CASCADE;

--
-- Constraints for table `tieBorrow`
--
ALTER TABLE `tieBorrow`
  ADD CONSTRAINT `tieBorrow_ibfk_1` FOREIGN KEY (`member`) REFERENCES `member` (`email`) ON DELETE NO ACTION ON UPDATE CASCADE,
  ADD CONSTRAINT `tieBorrow_ibfk_2` FOREIGN KEY (`tie`) REFERENCES `tie` (`id`) ON DELETE CASCADE ON UPDATE CASCADE;

--
-- Constraints for table `transaction`
--
ALTER TABLE `transaction`
  ADD CONSTRAINT `transaction_ibfk_1` FOREIGN KEY (`memberID`) REFERENCES `member` (`email`) ON DELETE CASCADE ON UPDATE CASCADE,
  ADD CONSTRAINT `transaction_ibfk_2` FOREIGN KEY (`type`) REFERENCES `transacType` (`id`),
  ADD CONSTRAINT `transaction_ibfk_3` FOREIGN KEY (`semester`) REFERENCES `semester` (`semester`) ON DELETE SET NULL ON UPDATE CASCADE,
  ADD CONSTRAINT `transaction_ibfk_4` FOREIGN KEY (`choir`) REFERENCES `choir` (`id`);

--
-- Constraints for table `uniform`
--
ALTER TABLE `uniform`
  ADD CONSTRAINT `uniform_ibfk_1` FOREIGN KEY (`choir`) REFERENCES `choir` (`id`) ON DELETE CASCADE ON UPDATE CASCADE;

--
-- Constraints for table `variables`
--
ALTER TABLE `variables`
  ADD CONSTRAINT `variable_validSemester` FOREIGN KEY (`semester`) REFERENCES `semester` (`semester`) ON DELETE CASCADE ON UPDATE CASCADE;
COMMIT;

/*!40101 SET CHARACTER_SET_CLIENT=@OLD_CHARACTER_SET_CLIENT */;
/*!40101 SET CHARACTER_SET_RESULTS=@OLD_CHARACTER_SET_RESULTS */;
/*!40101 SET COLLATION_CONNECTION=@OLD_COLLATION_CONNECTION */;
