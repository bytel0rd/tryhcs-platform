create table institutions (
    id bigserial primary key,
    name varchar(255) not null,
    email varchar(100) not null unique,
    classification varchar(50) not null,
    setting varchar(50) not null,
    address varchar(100),
    town varchar(100),
    state varchar(50),
    created_by bigint not null default 0,
    logo varchar(255),
    workspace_code varchar(100) not null,
    compliance_status varchar(20) not null default 'PENDING', -- (VERIFIED, PENDING, REJECTED, SUBMITTED)

    shadow_id uuid not null unique default gen_random_uuid(),
    -- foreign_key to staffs table modified_at timestamptz default Now (),
    modified_at timestamptz not null default Now (),
    created_at timestamptz not null default Now (),
    deleted_at timestamptz
);


create table users (
    id bigserial primary key,
    mobile varchar(30) not null unique,
    password varchar(255) not null,
    failed_attempts int not null default 0,
    device_ids varchar(70) array not null  default array[]::varchar[],
    last_login_time timestamptz,

    shadow_id uuid not null unique default gen_random_uuid(),
    deleted_at timestamptz,
    modified_at timestamptz not null default Now (),
    created_at timestamptz not null default Now ()
);

create table staffs (
    id bigserial primary key,
    first_name varchar(70) not null,
    last_name varchar(70) not null,
    mobile varchar(30) not null unique,
    title varchar(70) not null,
    institution_id bigint,
    profile_image varchar(255),

        shadow_id uuid not null unique default gen_random_uuid(),
    deleted_at timestamptz,
    modified_at timestamptz not null default Now (),
    created_at timestamptz not null default Now (),
unique(institution_id, mobile)
);

CREATE TABLE departments (
    id BIGSERIAL PRIMARY KEY,
    name VARCHAR(100) NOT NULL,
    institution_id BIGINT NOT NULL,
    domain VARCHAR(100) NOT NULL,
    head_staff_id VARCHAR(40),
    staffs_ids JSONB NOT NULL DEFAULT '[]'::jsonb,  -- ✅ corrected here
    phone_no VARCHAR(30),

    shadow_id uuid not null unique default gen_random_uuid(),
    deleted_at TIMESTAMPTZ,
    modified_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);