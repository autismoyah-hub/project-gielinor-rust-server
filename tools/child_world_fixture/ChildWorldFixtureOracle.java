import java.nio.file.*;
import java.lang.reflect.*;
import java.util.*;
import io.netty.buffer.*;
import net.rsprot.crypto.cipher.StreamCipher;
import net.rsprot.protocol.common.client.OldSchoolClientType;
import net.rsprot.protocol.game.outgoing.info.ByteBufRecycler;
import net.rsprot.protocol.internal.game.outgoing.info.util.ZoneIndexStorage;
import net.rsprot.protocol.game.outgoing.info.worldentityinfo.*;
import net.rsprot.protocol.game.outgoing.worldentity.SetActiveWorldV2;
import net.rsprot.protocol.game.outgoing.codec.worldentity.SetActiveWorldV2Encoder;
import net.rsprot.protocol.game.outgoing.map.RebuildWorldEntityV4;
import net.rsprot.protocol.game.outgoing.map.util.RebuildRegionZone;
import net.rsprot.protocol.game.outgoing.codec.map.RebuildWorldEntityV4Encoder;
import net.rsprot.protocol.game.outgoing.codec.worldentity.extendedinfo.WorldEntityAvatarExtendedInfoDesktopWriter;

/** Original revision239 library control, independent of the Rust encoder. */
public final class ChildWorldFixtureOracle {
    static String hex(ByteBuf b) { return ByteBufUtil.hexDump(b); }
    static String encode(Object encoder, Object message) throws Exception {
        Method method=Arrays.stream(encoder.getClass().getMethods())
            .filter(m->m.getName().startsWith("encode-") && m.getParameterTypes()[2]==message.getClass())
            .findFirst().orElseThrow();
        ByteBuf b=Unpooled.buffer();
        try { method.invoke(encoder,(StreamCipher)()->0,b,message); return hex(b); }
        finally { b.release(); }
    }
    public static void main(String[] args) throws Exception {
        List<String> rows=new ArrayList<>();
        rows.add("active "+encode(new SetActiveWorldV2Encoder(),new SetActiveWorldV2(new SetActiveWorldV2.DynamicWorldType(1,0))));
        rows.add("rebuild "+encode(new RebuildWorldEntityV4Encoder(),new RebuildWorldEntityV4(0,0,2,2,
            (x,z,level)->level==0 ? new RebuildRegionZone(400+x,400+z,0,0) : null)));
        ZoneIndexStorage zones=new ZoneIndexStorage(16);
        WorldEntityAvatarRepository repository=new WorldEntityAvatarRepository(UnpooledByteBufAllocator.DEFAULT,zones,List.of(new WorldEntityAvatarExtendedInfoDesktopWriter()),()->{throw new AssertionError("No chat");},new WorldEntityMap());
        ByteBufRecycler recycler=new ByteBufRecycler();
        WorldEntityInfo info=new WorldEntityInfo(1,UnpooledByteBufAllocator.DEFAULT,OldSchoolClientType.DESKTOP,repository,zones,recycler);
        Method rootCoord=Arrays.stream(WorldEntityInfo.class.getMethods()).filter(m->m.getName().startsWith("updateRootCoord-")).findFirst().orElseThrow();
        rootCoord.invoke(info,(10419<<14)|106);
        Class<?> build=Class.forName("net.rsprot.protocol.game.outgoing.info.util.BuildArea");
        long area=(Long)build.getMethod("constructor-impl",int.class,int.class,int.class,int.class).invoke(null,1296,7,13,13);
        Method rootArea=Arrays.stream(WorldEntityInfo.class.getMethods()).filter(m->m.getName().startsWith("updateRootBuildArea-")).findFirst().orElseThrow();
        rootArea.invoke(info,area);
        WorldEntityAvatar avatar=repository.getOrAlloc(1,-1,-1,2,2,0,0,0,0,10423*128+64,0,110*128+64,0,0,512);
        avatar.precompute$osrs_239_model();
        info.updateWorldEntities$osrs_239_model();
        if (info.getAddedWorldEntityIndices().isEmpty()) throw new AssertionError("Nonempty child admission required");
        rows.add("spawn "+hex(info.backingBuffer()));
        info.postUpdate$osrs_239_model();avatar.postUpdate();
        // Simulate a subsequent protocol cycle without starting a server scheduler.
        avatar.setAllocateCycle$osrs_239_model(-1);
        avatar.precompute$osrs_239_model();
        info.updateWorldEntities$osrs_239_model();
        rows.add("retain "+hex(info.backingBuffer()));
        info.onDealloc();recycler.cycle();
        Files.write(Path.of(args[0]),rows);
        System.out.println(String.join("\n",rows));
    }
}
