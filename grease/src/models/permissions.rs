use anyhow::Context;

use crate::db_conn::DbConn;
use async_graphql::{Object, Context};

// Roles that can be held by members to grant permissions.
pub struct Role {
    /// The name of the role.
    pub name: String,
    /// Used for ordering the positions (e.g. President beforee Ombudsman).
    pub rank: i32,
    /// The maximum number of the position allowed to be held at once.
    /// If it is 0 or less, no maximum is enforced.
    pub max_quantity: i32,
}

impl Role {
    pub async fn load_all(conn: &DbConn) -> Result<Vec<Self>> {
        sqlx::query_as!(Self, "SELECT * FROM role ORDER BY rank").query_all(&mut *conn).await.into()
    }

    pub async fn load_for_member(email: &str, conn: &DbConn) -> Result<Vec<Self>> {
        sqlx::query_as!(Self, "SELECT * FROM role WHERE name IN (SELECT rank FROM member_role WHERE member = ?) ORDER BY rank", email).query_all(&mut *conn).await.into()
    }
}

#[derive(ComplexObject)]
pub struct MemberRole {
    #[graphql(skip)]
    pub member: String,
    /// The name of the role being held.
    pub role: String,
}

#[complex]
impl MemberRole {
    /// The member holding the role.
    pub async fn member(&self, ctx: &Context) -> Result<Member> {
        let conn = ctx.data_unchecked::<DbConn>();
        Member::load(&self.member, conn).await
    }

    @[GraphQL::Object]
  class MemberRole
    include GraphQL::ObjectType

    class_getter table_name = "member_role"

    DB.mapping({
      member: String,
      role:   String,
    })

    def initialize(@member : String, @role : String)
    end

    def self.current_officers
      CONN.query_all "SELECT * FROM #{@@table_name}", as: MemberRole
    end

    def self.member_has_role?(email, role_name)
      CONN.query_one? "SELECT * FROM #{@@table_name} WHERE member = ? AND role = ?",
        email, role_name, as: MemberRole
    end

    def add
      raise "Member already has that role" if MemberRole.member_has_role? @member, @role

      CONN.exec "INSERT INTO #{@@table_name} (member, role) VALUES (?, ?)",
        @member, @role
    end

    def remove
      raise "Member does not have that role" unless MemberRole.member_has_role? @member, @role

      CONN.exec "DELETE #{@@table_name} WHERE member = ? AND role = ?",
        @member, @role
    end

  end

  @[GraphQL::Object]
  class Permission
    include GraphQL::ObjectType

    class_getter table_name = "permission"

    @[GraphQL::Enum(name: "PermissionType")]
    enum Type
      STATIC
      EVENT

      def self.mapping
        {
          "STATIC" => STATIC,
          "EVENT"  => EVENT,
        }
      end

      def to_rs
        Type.mapping.invert[self].downcase
      end

      def self.from_rs(rs)
        val = rs.read
        permission_type = val.as?(String).try { |v| Type.mapping[v.upcase]? }
        permission_type || raise "Invalid permission type returned from database: #{val}"
      end

      def self.parse(val)
        Type.mapping[val]? || raise "Invalid permission type variant provided: #{val}"
      end
    end

    DB.mapping({
      name:        String,
      description: String?,
      type:        {type: Type, default: Type::STATIC, converter: Type},
    })

    def self.all
      CONN.query_all "SELECT * FROM #{@@table_name} ORDER BY name", as: Permission
    end

    @[GraphQL::Field(description: "The name of the permission")]
    def name : String
      @name
    end

    @[GraphQL::Field(description: "A description of what the permission entails")]
    def description : String?
      @description
    end

    @[GraphQL::Field(description: "Whether the permission applies to a type of event or generally")]
    def type : Models::Permission::Type
      @type
    end
  end

  @[GraphQL::Object]
  class RolePermission
    include GraphQL::ObjectType

    class_getter table_name = "role_permission"

    DB.mapping({
      id:         Int32,
      role:       String,
      permission: String,
      event_type: String?,
    })

    def self.all
      CONN.query_all "SELECT * FROM #{@@table_name}", as: RolePermission
    end

    def self.add(role, permission, event_type)
      CONN.exec "INSERT IGNORE INTO #{@@table_name} (role, permission, event_type) \
        VALUES (?, ?, ?)", role, permission, event_type
    end

    def self.remove(role, permission, event_type)
      CONN.exec "DELETE #{@@table_name} WHERE role = ? AND permission = ? \
        AND event_type = ?", role, permission, event_type
    end

    @[GraphQL::Field(description: "The ID of the role permission")]
    def id : Int32
      @id
    end

    @[GraphQL::Field(description: "The name of the role this junction refers to")]
    def role : String
      @role
    end

    @[GraphQL::Field(description: "The name of the permission the role is awarded")]
    def permission : String
      @permission
    end

    @[GraphQL::Field(description: "The type of event the permission optionally applies to")]
    def event_type : String?
      @event_type
    end
  end

  @[GraphQL::Object]
  class MemberPermission
    include GraphQL::ObjectType

    DB.mapping({
      name:       String,
      event_type: String?,
    })

    def initialize(@name : String, @event_type : String?)
    end

    def self.for_member(email)
      CONN.query_all "SELECT permission as name, event_type FROM #{RolePermission.table_name} \
        INNER JOIN #{MemberRole.table_name} ON #{RolePermission.table_name}.role = #{MemberRole.table_name}.role \
        WHERE #{MemberRole.table_name}.member = ?", email, as: MemberPermission
    end

    @[GraphQL::Field]
    def name : String
      @name
    end

    @[GraphQL::Field]
    def event_type : String?
      @event_type
    end
  end
end
