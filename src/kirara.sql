-- 用户表
CREATE TABLE `users`
(
    `id`         int(11)      NOT NULL AUTO_INCREMENT,
    `username`   varchar(255) NOT NULL COMMENT '用户名',
    `password`   varchar(255) NOT NULL COMMENT '密码',
    `nickname`   varchar(255) NOT NULL COMMENT '昵称',
    `email`      varchar(255) NOT NULL COMMENT '邮箱',
    `avatar`     varchar(255)          DEFAULT NULL COMMENT '头像',
    `status`     tinyint(1)   NOT NULL DEFAULT '0' COMMENT '账号状态 0：待验证 1：正常 2：禁用 3：删除',
    `gender`     tinyint(1)   NOT NULL DEFAULT '0' COMMENT '性别 0：未知 1：男 2：女',
    `created_at` timestamp(6) NOT NULL DEFAULT CURRENT_TIMESTAMP(6) COMMENT '创建时间',
    `updated_at` timestamp(6) NOT NULL DEFAULT CURRENT_TIMESTAMP(6) ON UPDATE CURRENT_TIMESTAMP(6) COMMENT '更新时间',
    PRIMARY KEY (`id`)
) ENGINE = InnoDB
  AUTO_INCREMENT = 2
  DEFAULT CHARSET = utf8mb4 COMMENT ='用户表';

-- 媒体文件上传表
CREATE TABLE `media`
(
    `id`         int(11)             NOT NULL AUTO_INCREMENT,
    `user_id`    int(11)             NOT NULL COMMENT '用户id',
    `post_id`    int(11)                      DEFAULT NULL COMMENT '帖子id',
    `sort`       int(11)                      DEFAULT NULL COMMENT '排序',
    `name`       varchar(255)        NOT NULL COMMENT '文件名',
    `path`       varchar(255)        NOT NULL COMMENT '文件路径',
    `mime_type`  varchar(255)        NOT NULL COMMENT '文件类型 mime type 例如：image/jpeg',
    `size`       bigint(20) UNSIGNED NOT NULL COMMENT '文件大小',
    `created_at` timestamp(6)        NOT NULL DEFAULT CURRENT_TIMESTAMP(6) COMMENT '创建时间',
    PRIMARY KEY (`id`)
) ENGINE = InnoDB
  AUTO_INCREMENT = 2
  DEFAULT CHARSET = utf8mb4 COMMENT ='媒体文件上传表';

-- 图片上传
CREATE TABLE `images`
(
    `id`         int(11)      NOT NULL AUTO_INCREMENT,
    `user_id`    int(11)      NOT NULL COMMENT '用户id',
    `name`       varchar(255) NOT NULL COMMENT '文件名',
    `path`       varchar(255) NOT NULL COMMENT '文件路径',
    `mime_type`  varchar(255) NOT NULL COMMENT '文件类型 mime type 例如：image/jpeg',
    `size`       bigint(20)   NOT NULL COMMENT '文件大小',
    `width`      int(11)      NOT NULL COMMENT '宽度',
    `height`     int(11)      NOT NULL COMMENT '高度',
    `created_at` timestamp(6) NOT NULL DEFAULT CURRENT_TIMESTAMP(6) COMMENT '创建时间',
    PRIMARY KEY (`id`)
) ENGINE = InnoDB
  AUTO_INCREMENT = 2
  DEFAULT CHARSET = utf8mb4 COMMENT ='图片上传';

-- 用户登录日志表
CREATE TABLE `user_login_logs`
(
    `id`         int(11)      NOT NULL AUTO_INCREMENT,
    `user_id`    int(11)      NOT NULL COMMENT '用户id',
    `ip`         varchar(255) NOT NULL COMMENT 'ip地址',
    `user_agent` varchar(255) NOT NULL COMMENT 'user_agent',
    `created_at` timestamp(6) NOT NULL DEFAULT CURRENT_TIMESTAMP(6) COMMENT '创建时间',
    PRIMARY KEY (`id`)
) ENGINE = InnoDB
  AUTO_INCREMENT = 2
  DEFAULT CHARSET = utf8mb4 COMMENT ='用户登录日志表';

-- 用户访问日志表
CREATE TABLE `user_visit_logs`
(
    `id`         int(11)      NOT NULL AUTO_INCREMENT,
    `user_id`    int(11)      NOT NULL COMMENT '用户id',
    `ip`         varchar(255) NOT NULL COMMENT 'ip地址',
    `user_agent` varchar(255) NOT NULL COMMENT 'user_agent',
    `path`       varchar(255) NOT NULL COMMENT '访问路径',
    `method`     varchar(255) NOT NULL COMMENT '请求方法',
    `input`      text         NOT NULL COMMENT '请求参数',
    `status`     tinyint(1)   NOT NULL DEFAULT '0' COMMENT '状态 0：失败 1：成功',
    `created_at` timestamp(6) NOT NULL DEFAULT CURRENT_TIMESTAMP(6) COMMENT '创建时间',
    PRIMARY KEY (`id`)
) ENGINE = InnoDB
  AUTO_INCREMENT = 2
  DEFAULT CHARSET = utf8mb4 COMMENT ='用户访问日志表';

-- 帖子表
CREATE TABLE `posts`
(
    `id`          int(11)      NOT NULL AUTO_INCREMENT,
    `user_id`     int(11)      NOT NULL COMMENT '用户id',
    #            帖子描述 可选
    `description` varchar(255)          DEFAULT NULL COMMENT '帖子描述',
    `status`      tinyint(1)   NOT NULL DEFAULT '0' COMMENT '帖子状态 0：待审核 1：正常 2：禁用 3：删除',
    `created_at`  timestamp(6) NOT NULL DEFAULT CURRENT_TIMESTAMP(6) COMMENT '创建时间',
    `updated_at`  timestamp(6) NOT NULL DEFAULT CURRENT_TIMESTAMP(6) ON UPDATE CURRENT_TIMESTAMP(6) COMMENT '更新时间',
    PRIMARY KEY (`id`)
) ENGINE = InnoDB
  AUTO_INCREMENT = 2
  DEFAULT CHARSET = utf8mb4 COMMENT ='帖子表';
