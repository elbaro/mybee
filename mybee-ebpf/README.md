bool dispatch_command(THD *thd, const COM_DATA *com_data,
                      enum enum_server_command command);

int mysql_execute_command(THD *thd, bool first_level = false);

bool do_command(THD *thd);


bool parse_sql(THD *thd, Parser_state *parser_state,
               Object_creation_ctx *creation_ctx);


do_command (read connection and exec 1 cmd) => dispatch_command


<!-- https://github.com/mysql/mysql-server/blob/1bfe02bdad6604d54913c62614bde57a055c8332/sql/sql_parse.cc -->
dispatch_command(thd, com_data, command)
    - COM_STMT_EXECUTE (exec prepared one): mysql_stmt_precheck + mysqld_stmt_execute
    - COM_QUERY:
        - thd->m_digest is filled
        - alloc_query(thd, com_data->com_query.query, com_data->com_query.length)
        - char *packet_end = thd->query().str + thd->query().length;

        - dispatch_sql_command (parse AST)
            - parse_sql
            - mysql_execute_command

        void dispatch_sql_command(THD *thd, Parser_state *parser_state) 
        bool parse_sql(THD *thd, Parser_state *parser_state,
               Object_creation_ctx *creation_ctx)
        int mysql_execute_command(THD *thd, bool first_level) - called multiple times
        Parser_state has text sql


        `thd->set_query(thd->query().str, qlen - 1);`

              thd->set_query_for_display(thd->rewritten_query().ptr(),
                                 thd->rewritten_query().length());


          // we produce digest if it's not explicitly turned off
        // by setting maximum digest length to zero
        if (get_max_digest_length() != 0)
            parser_state->m_input.m_compute_digest = true;



            <!-- struct COM_QUERY_DATA {
            const char *query;
            unsigned int length;
            PS_PARAM *parameters;
            unsigned long parameter_count;
            }; -->