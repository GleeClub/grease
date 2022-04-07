module Input
  @[GraphQL::InputObject]
  class NewMember
    include GraphQL::InputObjectType

    @enrollment : Models::ActiveSemester::Enrollment?

    getter email, first_name, preferred_name, last_name,
      pass_hash, phone_number, picture, passengers, location,
      on_campus, about, major, minor, hometown, arrived_at_tech,
      gateway_drug, conflicts, dietary_restrictions,
      enrollment, section

    @[GraphQL::Field]
    def initialize(
      @email : String,
      @first_name : String,
      @preferred_name : String?,
      @last_name : String,
      @pass_hash : String,
      @phone_number : String,
      @picture : String?,
      @passengers : Int32,
      @location : String,
      @on_campus : Bool?,
      @about : String?,
      @major : String?,
      @minor : String?,
      @hometown : String?,
      @arrived_at_tech : Int32?,
      @gateway_drug : String?,
      @conflicts : String?,
      @dietary_restrictions : String,
      enrollment : String?,
      @section : String?
    )
      @enrollment = enrollment.try { |e| Models::ActiveSemester::Enrollment.parse e }
    end
  end

  @[GraphQL::InputObject]
  class RegisterForSemesterForm
    include GraphQL::InputObjectType

    @enrollment : Models::ActiveSemester::Enrollment

    getter location, on_campus, conflicts,
      dietary_restrictions, enrollment, section

    @[GraphQL::Field]
    def initialize(
      @location : String,
      @on_campus : Bool?,
      @conflicts : String,
      @dietary_restrictions : String,
      enrollment : String,
      @section : String
    )
      @enrollment = Models::ActiveSemester::Enrollment.parse enrollment
    end
  end

  @[GraphQL::InputObject]
  class NewEvent
    include GraphQL::InputObjectType

    getter event, gig, repeat

    @[GraphQL::Field]
    def initialize(@event : Input::NewEventFields, @gig : Input::NewGig?,
                   @repeat : Input::NewEventPeriod?)
    end
  end

  @[GraphQL::InputObject]
  class NewEventFields
    include GraphQL::InputObjectType

    @call_time : Time
    @release_time : Time?

    getter name, semester, type, call_time,
      release_time, points, comments,
      location, gig_count, default_attend

    @[GraphQL::Field]
    def initialize(
      @name : String,
      @semester : String,
      @type : String,
      call_time : String,
      release_time : String?,
      @points : Int32,
      @comments : String?,
      @location : String?,
      @gig_count : Bool?,
      @default_attend : Bool
    )
      @call_time = Input.parse_datetime call_time, "Call time"
      @release_time = release_time.try { |t| Input.parse_datetime t, "Release time" }
    end
  end

  @[GraphQL::InputObject]
  class NewGig
    include GraphQL::InputObjectType

    @performance_time : Time

    getter performance_time, uniform, contact_name,
      contact_email, contact_phone, price,
      public, summary, description

    @[GraphQL::Field]
    def initialize(
      performance_time : String,
      @uniform : Int32,
      @contact_name : String?,
      @contact_email : String?,
      @contact_phone : String?,
      @price : Int32?,
      @public : Bool,
      @summary : String?,
      @description : String?
    )
      @performance_time = Input.parse_datetime performance_time, "Performance time"
    end
  end
