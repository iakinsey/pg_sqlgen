CREATE SCHEMA IF NOT EXISTS sqlgen AUTHORIZATION current_user;
CREATE SCHEMA sqlgen_internal AUTHORIZATION current_user;
REVOKE ALL ON SCHEMA sqlgen_internal FROM public;
