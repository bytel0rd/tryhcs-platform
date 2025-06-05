create table corporate_compliance (
    id bigserial primary key,
    institution_id bigint not null,
    rc_no varchar(50) not null,
    tin varchar(50) not null,
    private_healthcare_certificate_url varchar(255),

    corporate_account_number varchar(30) not null,  
    corporate_bank_code varchar(30) not null,

    stage varchar(20) not null default 'PENDING', -- (VERIFIED, PENDING, REJECTED, SUBMITTED)

    created_by varchar(40) not null,
    deleted_at timestamptz,
    modified_at timestamptz not null default Now (),
    created_at timestamptz not null default Now ()
);

create table healthcare_compliance (
    id bigserial primary key,
    institution_id bigint not null,

    licensed_medical_doctor_name varchar(100) not null,  
    licensed_medical_doctor_mdcn_no varchar(100) not null,  
    licensed_medical_doctor_mdcn_speciality varchar(255) not null,  
    licensed_medical_doctor_mdcn_image_url varchar(255) not null,
    licensed_medical_doctor_email varchar(255) not null,
    licensed_medical_doctor_phone_no varchar(255) not null,

    stage varchar(20) not null default 'PENDING', -- (VERIFIED, PENDING, REJECTED, SUBMITTED)

    created_by varchar(40) not null,
    deleted_at timestamptz,
    modified_at timestamptz not null default Now (),
    created_at timestamptz not null default Now ()
);

create table financial_compliance (
    id bigserial primary key,
    institution_id bigint not null,

    director_legal_name varchar(100) not null,  
    director_legal_bvn varchar(15) not null,  
    director_legal_dob varchar(10) not null,  -- validate bvn endpoint!
    director_legal_gov_id_type varchar(255) not null,  
    director_legal_gov_id_url varchar(255) not null,  

    stage varchar(20) not null default 'PENDING', -- (VERIFIED, PENDING, REJECTED, SUBMITTED)

    created_by varchar(40) not null,
    deleted_at timestamptz,
    modified_at timestamptz not null default Now (),
    created_at timestamptz not null default Now ()
);


create table staff_compliance (
    id bigserial primary key,
    staff_id bigint not null,

    license_type varchar(50) not null, -- (MDCN, PCN, NMCN, RRBN, MLSCN) 
    license_no varchar(100),  
    license_certificate_url varchar(255),  

    stage varchar(20) not null default 'PENDING', 

    created_by varchar(40) not null,
    deleted_at timestamptz,
    modified_at timestamptz not null default Now (),
    created_at timestamptz not null default Now ()
);