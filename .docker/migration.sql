CREATE TABLE todos (
  id uuid DEFAULT uuid_generate_v4(),
  name VARCHAR NOT NULL,
  description VARCHAR NOT NULL,
  created_at timestamptz DEFAULT NOW() NOT NULL,
  updated_at timestamptz DEFAULT NOW() NOT NULL,
  deleted_at timestamptz,
  CONSTRAINT todos_pkey PRIMARY KEY(id)
);