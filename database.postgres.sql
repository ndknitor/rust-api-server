\c etdb;

CREATE TABLE "user" (
    user_id INT NOT NULL,
    email VARCHAR(128) NOT NULL,
    fullname VARCHAR(128) NOT NULL DEFAULT '',
    phone VARCHAR(16) NOT NULL DEFAULT '',
    address VARCHAR(128) NOT NULL DEFAULT '',
    role_id INT NOT NULL DEFAULT 0,
    password VARCHAR(128) NOT NULL DEFAULT '',
    PRIMARY KEY (user_id)
);

CREATE TABLE bus (
    bus_id INT NOT NULL,
    name VARCHAR(128),
    license_plate VARCHAR(16) NOT NULL,
    deleted BOOLEAN NOT NULL DEFAULT FALSE,
    PRIMARY KEY (bus_id)
);

CREATE TABLE route (
    route_id INT NOT NULL,
    from_location VARCHAR(128) NOT NULL DEFAULT '',
    to_location VARCHAR(128) NOT NULL DEFAULT '',
    base_price INT NOT NULL,
    deleted BOOLEAN NOT NULL DEFAULT FALSE,
    PRIMARY KEY (route_id)
);

CREATE TABLE trip (
    trip_id INT NOT NULL,
    route_id INT NOT NULL,
    start_date TIMESTAMP NOT NULL,
    end_date TIMESTAMP NOT NULL,
    bus_id INT NOT NULL,
    PRIMARY KEY (trip_id),
    FOREIGN KEY (route_id) REFERENCES route (route_id),
    FOREIGN KEY (bus_id) REFERENCES bus (bus_id)
);

CREATE TABLE seat (
    seat_id INT NOT NULL,
    bus_id INT NOT NULL,
    price INT NOT NULL DEFAULT 0,
    deleted BOOLEAN NOT NULL DEFAULT FALSE,
    name VARCHAR(128) NOT NULL,
    PRIMARY KEY (seat_id),
    FOREIGN KEY (bus_id) REFERENCES bus (bus_id)
);

CREATE TABLE ticket (
    ticket_id INT NOT NULL,
    status INT NOT NULL DEFAULT 0,
    price INT NOT NULL DEFAULT 0,
    trip_id INT NOT NULL,
    seat_id INT NOT NULL,
    booked_date TIMESTAMP NOT NULL,
    user_id INT NOT NULL,
    from_location VARCHAR(128),
    to_location VARCHAR(128),
    PRIMARY KEY (ticket_id),
    FOREIGN KEY (trip_id) REFERENCES trip (trip_id),
    FOREIGN KEY (seat_id) REFERENCES seat (seat_id),
    FOREIGN KEY (user_id) REFERENCES "user" (user_id)
);

-- Computed seat count: use a view or query instead of a computed column
CREATE OR REPLACE FUNCTION get_seat_count_for_bus(p_bus_id INT)
RETURNS INT
LANGUAGE sql
STABLE
AS $$
    SELECT COUNT(*)::INT FROM seat WHERE bus_id = p_bus_id;
$$;

CREATE VIEW bus_with_seat_count AS
SELECT b.*, get_seat_count_for_bus(b.bus_id) AS seat_count
FROM bus b;

-- Seed data

INSERT INTO "user" (user_id, email, fullname, phone, address, role_id, password) VALUES
(0, 'ngodinhkhoinguyen69@gmail.com', 'Ngo Dinh Khoi Nguyen', '349-548-8233', '69 Asplie Lica', 0, '123456'),
(1, 'akenewel0@gravatar.com', 'Ainsley Kenewel', '521-396-4437', '8 Alpine Junction', 3, '123456'),
(2, 'dguidi1@google.it', 'Dannie Guidi', '834-807-3579', '3134 Talisman Avenue', 2, '123456'),
(3, 'greppaport2@bloglines.com', 'Gypsy Reppaport', '178-549-8246', '6 Rockefeller Trail', 0, '123456'),
(4, 'gsercombe3@jiathis.com', 'Gill Sercombe', '210-267-1214', '103 Montana Crossing', 0, '123456'),
(5, 'wchestney4@time.com', 'Whitaker Chestney', '765-243-8331', '57 Lakeland Court', 3, '123456'),
(6, 'jpease5@networksolutions.com', 'Justin Pease', '648-570-8710', '398 Sunbrook Parkway', 3, '123456'),
(7, 'amadgin6@amazon.de', 'Ayn Madgin', '854-464-2842', '39843 Talisman Hill', 0, '123456'),
(8, 'rhalleday7@economist.com', 'Rex Halleday', '541-709-3920', '56 Crescent Oaks Place', 0, '123456'),
(9, 'fgonnel8@wiley.com', 'Frederigo Gonnel', '750-973-2099', '74 Karstens Trail', 3, '123456'),
(10, 'jpeirazzi9@yahoo.co.jp', 'Juliana Peirazzi', '391-480-0322', '50 Lyons Park', 3, '123456');

INSERT INTO bus (bus_id, name, license_plate, deleted) VALUES
(1, 'Sdasad', '41A23435', FALSE),
(2, 'Sdasad', '43278322', FALSE),
(3, 'Sdasad', '23434322', FALSE),
(4, 'Sdasad', '23423234', FALSE),
(5, 'Sdasad', '23423423', FALSE),
(7, 'Sdasad', '34534532', FALSE),
(10, 'Sdasad', '34534534', FALSE),
(11, 'Sdasad', '23425464', FALSE),
(12, 'Sdasad', '34657432', FALSE),
(13, 'Sdasad', '43543434', FALSE),
(14, 'From the nam', '47A82783', FALSE);

INSERT INTO route (route_id, from_location, to_location, base_price, deleted) VALUES
(1, 'Can Tho', 'Ca Mau', 10000, FALSE),
(2, 'Can Tho', 'Singapore', 10000, FALSE),
(3, 'Can Tho', 'Can Tho', 10000, FALSE),
(4, 'Can Tho', 'Can Tho', 10000, FALSE),
(5, 'Can Tho', 'Can Tho', 10000, FALSE),
(6, 'Can Tho', 'Can Tho', 10000, FALSE),
(7, 'Can Tho', 'Can Tho 2', 10000, FALSE),
(8, 'Can Tho', 'Can Tho', 10000, TRUE),
(9, 'Can Tho', 'Can Tho', 10000, TRUE),
(10, 'Can Tho', 'Can Tho', 10000, TRUE),
(11, 'Can Tho', 'Can Tho', 10000, FALSE),
(12, 'Can Tho', 'Can Tho 3', 10000, FALSE),
(13, 'Can Tho', 'Can Tho', 10000, TRUE),
(14, 'Can Tho', 'Can Tho', 10000, FALSE),
(15, 'Can Tho', 'Can Tho', 10000, FALSE),
(16, 'Can Tho', 'Can Tho', 10000, FALSE),
(17, 'Can Tho', 'Can Tho', 10000, FALSE),
(18, 'Can Tho', 'Ca Mau', 500000, FALSE),
(19, 'Can Tho', 'Ca Mau', 200000, FALSE),
(20, 'Can Tho', 'Can Tho', 10000, TRUE),
(21, 'Long An', 'Ho Chi Minh', 2000, TRUE),
(22, 'Long An', 'Ha Noi', 500000, TRUE),
(23, 'Long An', 'Ho Chi Minh', 500000, TRUE);

INSERT INTO seat (seat_id, bus_id, price, deleted, name) VALUES
(21, 1, 250000, FALSE, 'A1'),
(22, 1, 250000, FALSE, 'A2'),
(23, 1, 250000, FALSE, 'A3'),
(24, 1, 250000, FALSE, 'A4'),
(25, 1, 450000, FALSE, 'A5'),
(26, 1, 250000, FALSE, 'A6'),
(27, 1, 250000, FALSE, 'A7'),
(28, 1, 250000, FALSE, 'A8'),
(29, 1, 250000, FALSE, 'A9'),
(30, 1, 250000, FALSE, 'A10'),
(31, 1, 250000, FALSE, 'A11'),
(32, 2, 26000, FALSE, 'A11'),
(33, 2, 250000, FALSE, 'A11'),
(34, 2, 250000, FALSE, 'A11'),
(35, 2, 250000, FALSE, 'A11'),
(56, 4, 26000, FALSE, 'A11'),
(57, 4, 26000, FALSE, 'A11'),
(58, 4, 26000, FALSE, 'A11'),
(71, 1, 250000, FALSE, 'B11'),
(72, 1, 250000, FALSE, 'B11'),
(73, 1, 250000, FALSE, 'B11'),
(74, 1, 250000, FALSE, 'B11'),
(75, 1, 250000, FALSE, 'B11'),
(76, 1, 250000, FALSE, 'B11'),
(77, 1, 250000, FALSE, 'B11'),
(97, 2, 250000, FALSE, 'B11'),
(98, 2, 250000, FALSE, 'B11'),
(99, 2, 250000, FALSE, 'B11'),
(100, 2, 250000, FALSE, 'B11'),
(101, 2, 250000, FALSE, 'B11'),
(102, 2, 250000, FALSE, 'B11'),
(103, 2, 250000, FALSE, 'B11'),
(104, 2, 250000, FALSE, 'B11'),
(105, 2, 250000, FALSE, 'B11'),
(106, 2, 250000, FALSE, 'B11'),
(107, 2, 250000, FALSE, 'B11'),
(128, 2, 250000, FALSE, 'B11'),
(129, 2, 250000, FALSE, 'B11'),
(130, 2, 250000, FALSE, 'B11'),
(131, 2, 250000, FALSE, 'B11'),
(132, 2, 250000, FALSE, 'B11'),
(133, 2, 250000, FALSE, 'B11'),
(134, 14, 300000, FALSE, 'A1'),
(135, 14, 300000, FALSE, 'A2'),
(136, 14, 300000, FALSE, 'A3'),
(137, 14, 300000, FALSE, 'A4'),
(138, 14, 300000, FALSE, 'A5'),
(139, 14, 300000, FALSE, 'A6'),
(140, 14, 300000, FALSE, 'A7'),
(141, 14, 300000, FALSE, 'A8'),
(142, 14, 300000, FALSE, 'A9'),
(143, 14, 300000, FALSE, 'A10'),
(144, 14, 300000, FALSE, 'A11'),
(145, 14, 300000, FALSE, 'A12'),
(146, 14, 300000, FALSE, 'A13'),
(147, 14, 300000, FALSE, 'A14'),
(148, 14, 300000, FALSE, 'A15'),
(149, 14, 300000, FALSE, 'A16'),
(150, 14, 300000, FALSE, 'A17'),
(151, 14, 300000, FALSE, 'A18'),
(152, 14, 300000, FALSE, 'A19'),
(153, 14, 300000, FALSE, 'A20'),
(154, 14, 300000, FALSE, 'A21'),
(155, 14, 300000, FALSE, 'A22'),
(156, 14, 300000, FALSE, 'A23'),
(157, 14, 300000, FALSE, 'A24'),
(158, 14, 300000, FALSE, 'A25'),
(159, 14, 300000, FALSE, 'A26'),
(160, 14, 300000, FALSE, 'A27'),
(161, 14, 300000, FALSE, 'A28'),
(162, 14, 300000, FALSE, 'A29'),
(163, 14, 300000, FALSE, 'A30');

INSERT INTO trip (trip_id, route_id, start_date, end_date, bus_id) VALUES
(2, 2, '2022-11-13 08:14:27', '2022-11-15 08:14:27', 2),
(3, 2, '2022-11-15 21:00:00', '2022-11-15 22:00:00', 2),
(4, 2, '2022-11-15 19:00:00', '2022-11-15 22:00:00', 3),
(5, 2, '2022-11-15 18:00:00', '2022-11-15 22:00:00', 4),
(6, 2, '2022-11-15 17:00:00', '2022-11-15 22:00:00', 5),
(35, 2, '2022-11-15 20:00:00', '2022-11-15 22:00:00', 3),
(36, 2, '2022-11-22 06:49:02', '2022-11-22 11:49:02', 14),
(37, 17, '2022-10-30 15:35:47', '2022-11-08 15:35:47', 14);

